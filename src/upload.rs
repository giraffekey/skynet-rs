use crate::{SkynetClient, SkynetError::*, SkynetResult, util::make_uri, URI_SKYNET_PREFIX};
use std::{
  collections::HashMap,
  fs,
  io::Write,
  path::Path,
  str,
};
use hyper::{body, Request};
use mime::Mime;
use serde::Deserialize;
use textnonce::TextNonce;
use walkdir::WalkDir;
use tus_async_client::{Client, HttpHandler};
use reqwest::{self, ClientBuilder};
use std::rc::Rc;
use std::sync::Arc;
use http::Uri;
use crate::util::make_reqwest_headers;

/// Skynet uploads data in chunks.
/// The size of these chunks depends on erasure coding settings specified for the fanout and the specified encryption type.
/// The formula for the size of these chunks is chunkSize := (4MiB — encryptionOverhead) * fanoutDataPieces.
/// By default, data uploaded to Skynet uses 10 data pieces for its fanout and Threefish for encryption which doesn’t have any overhead.
/// As a result, the default chunk size is 40MiB. Since portals have limited amounts of RAM,
/// they can’t keep these chunks in memory while waiting for users to resume their uploads.
/// That’s why the chunk size specified in TUS needs to be a multiple of the Skynet chunk size. As long as they match,
/// the portal can upload the chunks and free up memory while waiting for more data.
const SKYNET_TUS_CHUNK_SIZE : u64 = (1 << 22) * 10;

/// The size at which files are considered "large" and will be uploaded using the tus resumable upload protocol. This is the size of one chunk by default (40 mib). Note that this does not affect the actual size of chunks used by the protocol.
const USE_TUS_THRESHOLD_BYTES : u64 = SKYNET_TUS_CHUNK_SIZE;

#[derive(Debug, Clone)]
pub struct UploadOptions {
  pub endpoint_path: String,
  pub api_key: Option<String>,
  pub custom_user_agent: Option<String>,
  pub portal_file_fieldname: String,
  pub portal_directory_file_fieldname: String,
  pub custom_filename: Option<String>,
  pub custom_dirname: Option<String>,
  pub skykey_name: Option<String>,
  pub skykey_id: Option<String>,
}

impl Default for UploadOptions {
  fn default() -> Self {
  	Self {
      endpoint_path: "/skynet/skyfile".to_string(),
      api_key: None,
      custom_user_agent: None,
      portal_file_fieldname: "file".to_string(),
      portal_directory_file_fieldname: "files[]".to_string(),
      custom_filename: None,
      custom_dirname: None,
      skykey_name: None,
      skykey_id: None,
    }
  }
}

#[derive(Deserialize)]
struct UploadResponse {
  skylink: String,
}

pub async fn upload_data(
  client: &SkynetClient,
  data: HashMap<String, (Mime, Vec<u8>)>,
  opt: UploadOptions,
) -> SkynetResult<String> {
  let req = Request::builder().method("POST");

  let mut query = HashMap::new();

  let (fieldname, filename) =
    if data.len() == 1 && opt.custom_dirname.is_none() {
      (opt.portal_file_fieldname.clone(), "".to_string())
    } else {
      if let Some(ref custom_dirname) = opt.custom_dirname {
        (opt.portal_directory_file_fieldname.clone(), custom_dirname.clone())
      } else {
        return Err(NoCustomDirname);
      }
    };

  if !filename.is_empty() {
    query.insert("filename".into(), filename);
  }

  if let Some(ref skykey_name) = opt.skykey_name {
    query.insert("skykeyname".into(), skykey_name.clone());
  }

  if let Some(ref skykey_id) = opt.skykey_id {
    query.insert("skykeyid".into(), skykey_id.clone());
  }

  let mut body = Vec::new();
  let boundary = TextNonce::sized(68).map_err(TextNonceError)?.into_string().into_bytes();

  for (filename, (mime, bytes)) in &data {
    let disposition = format!("form-data; name=\"{}\"; filename=\"{}\"", fieldname, filename);
    let headers = format!("Content-Disposition: {}\r\nContent-Type: {}\r\n", disposition, mime);

    body.write_all(b"--").map_err(WriteError)?;
    body.write_all(&boundary).map_err(WriteError)?;
    body.write_all(b"\r\n").map_err(WriteError)?;
    body.write_all(headers.as_bytes()).map_err(WriteError)?;
    body.write_all(b"\r\n").map_err(WriteError)?;
    body.write_all(bytes).map_err(WriteError)?;
    body.write_all(b"\r\n").map_err(WriteError)?;
  }

  body.write_all(b"--").map_err(WriteError)?;
  body.write_all(&boundary).map_err(WriteError)?;
  body.write_all(b"--\r\n").map_err(WriteError)?;

  let content_type = format!(
    "{}; boundary=\"{}\"",
    mime::MULTIPART_FORM_DATA,
    str::from_utf8(&boundary).map_err(Utf8Error)?);

  let uri = make_uri(
    client.get_portal_url(),
    opt.endpoint_path,
    opt.api_key.clone(),
    None,
    query);

  let mut req = req
    .uri(uri)
    .header("Content-Type", content_type);

  if let Some(apikey) = &opt.api_key.or(client.get_options().api_key.clone()) {
    req = req.header("Skynet-Api-Key", apikey.clone());
  }

  if let Some(custom_user_agent) = opt.custom_user_agent {
    req = req.header("User-Agent", custom_user_agent);
  }

  let req = req.body(body.into()).map_err(HttpError)?;
  let res = client.http.request(req).await.map_err(HyperError)?;
  let body = body::to_bytes(res.into_body()).await.map_err(HyperError)?;
  let body_str = str::from_utf8(&body).map_err(Utf8Error)?;
  let res: UploadResponse = serde_json::from_str(body_str)
    .map_err(|_| PortalResponse(body_str.to_string()))?;

  // disabled since the Skynet api itself doesn't return it iike this anymore
  // let skylink = format!("{}{}", URI_SKYNET_PREFIX, res.skylink);

  Ok(res.skylink)
}

pub fn upload_data_tus_headers(
  client: &SkynetClient,
  path: &Path,
  opt: &UploadOptions,
) -> SkynetResult<HashMap<String, String>> {
  let mut headers = HashMap::new();

  if let Some(apikey) = &opt.api_key.clone().or(client.get_options().api_key.clone()) {
    headers.insert("Skynet-API-Key".to_string(), apikey.clone());
  }

  if let Some(custom_user_agent) = opt.custom_user_agent.clone() {
    headers.insert("User-Agent".to_string(), custom_user_agent);
  }

  Ok(headers)
}

pub fn upload_data_query_params(
  client: &SkynetClient,
  path: &Path,
  opt: &UploadOptions,
) -> SkynetResult<HashMap<String, String>> {
  let filename =
      if opt.custom_dirname.is_none() {
        "".to_string()
      } else {
        if let Some(ref custom_dirname) = opt.custom_dirname {
          custom_dirname.clone()
        } else {
          return Err(NoCustomDirname);
        }
      };

  let mut query = HashMap::new();

  if !filename.is_empty() {
    query.insert("filename".into(), filename);
  }

  if let Some(ref skykey_name) = opt.skykey_name {
    query.insert("skykeyname".into(), skykey_name.clone());
  }

  if let Some(ref skykey_id) = opt.skykey_id {
    query.insert("skykeyid".into(), skykey_id.clone());
  }

  Ok(query)
}

pub fn upload_data_tus_uri(
  client: &SkynetClient,
  path: &Path,
  opt: &UploadOptions,
) -> SkynetResult<Uri> {
  Ok(make_uri(
    client.get_portal_url(),
    "/skynet/tus".to_string(),
    opt.api_key.clone(),
    None,
    upload_data_query_params(client, path, opt)?))
}

pub fn create_tus_client(
  client: &SkynetClient,
  path: &Path,
  opt: UploadOptions,
) -> SkynetResult<tus_async_client::Client> {
  let headers = make_reqwest_headers(
    upload_data_tus_headers(
      &client,
      path,
      &opt.clone())?);

  let req = reqwest::Client::builder()
      .default_headers(headers.clone());

  Ok(Client::new(
    HttpHandler::new(
      Arc::new(req
          .build()
          .map_err(ReqwestError)?
      ))))
}

pub async fn tus_create_upload_url(
  client: &SkynetClient,
  path: &Path,
  opt: UploadOptions,
) -> SkynetResult<String> {
  let uri = upload_data_tus_uri(
    &client,
    path,
    &UploadOptions::default()
  )?;

  create_tus_client(client, path, opt)?
      .create(&uri.to_string(), path)
      .await
      .map_err(TUSError)
}

pub async fn upload_data_tus(
  client: &SkynetClient,
  path: &Path,
  opt: UploadOptions,
) -> SkynetResult<String> {
  let upload_url = tus_create_upload_url(client, path, opt.clone()).await?;
  let tus_client = create_tus_client(client, path, opt.clone())?;

  // perform upload
  tus_client
      .upload_with_chunk_size(&upload_url, path, SKYNET_TUS_CHUNK_SIZE as usize)
      .await
      .map_err(TUSError)?
  ;

  // finish upload and retrieve skylink
  get_tus_upload_skylink(client, path, opt.clone(), upload_url).await
}

/// get skylink from HEAD request headers after all pieces finished upload
pub async fn get_tus_upload_skylink(
  client: &SkynetClient,
  path: &Path,
  opt: UploadOptions,
  upload_url: String
) -> SkynetResult<String> {
  let headers = upload_data_tus_headers(&client, path, &opt)?;
  let headers = make_reqwest_headers(headers);

  let meta = reqwest::Client::new()
      .head(upload_url)
      .headers(headers)
      .header("tus-resumable", "1.0.0")
      .send()
      .await
      .map_err(ReqwestError)?;

  let headers = meta
      .headers();

  let skylink = headers
      .get("skynet-skylink")
      .expect("skylink header not found")
      .to_str()
      .expect("failed to parse skylink header to string");

  Ok(skylink.to_string())
}

pub async fn upload_file(
  client: &SkynetClient,
  path: &Path,
  opt: UploadOptions,
) -> SkynetResult<String> {
  if !path.is_file() {
    return Err(NotFile);
  }

  let mime = mime_guess::from_path(path)
      .first()
      .unwrap_or(mime::APPLICATION_OCTET_STREAM);

  // "Large file uploads are automatically supported in skynet-js and skynet-nodejs.
  //  Any file over 40MB will automatically use the built-in tus upload client."
  //   - https://docs.skynetlabs.com/integrations/resumable-uploads-using-tus
  if fs::metadata(path).map_err(FileError)?.len() >= USE_TUS_THRESHOLD_BYTES {
    upload_data_tus(client, path, opt).await
  }

  // load data in mem and send
  else {
    let bytes = fs::read(path)
        .map_err(FileError)?;

    let filename = path
        .file_name()
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();

    let mut data = HashMap::new();
    data.insert(filename, (mime, bytes));

    upload_data(client, data, opt).await
  }
}

pub async fn upload_directory(
  client: &SkynetClient,
  path: &Path,
  opt: UploadOptions,
) -> SkynetResult<String> {
  if !path.is_dir() {
    return Err(NotDirectory);
  }

  let mut data = HashMap::new();
  let dirpath = path;

  for entry in WalkDir::new(dirpath) {
    let entry = entry.unwrap();
    let path = entry.path();
    if path.is_file() {
      let filename = path.as_os_str().to_str().unwrap().to_string();
      let mime = if let Some(mime) = mime_guess::from_path(path).first() {
        mime
      } else {
        mime::APPLICATION_OCTET_STREAM
      };
      let bytes = fs::read(path).map_err(FileError)?;

      data.insert(filename, (mime, bytes));
    }
  }

  let dirname = path.file_name().unwrap().to_str().unwrap().to_string();

  let opt = UploadOptions {
    custom_dirname: Some(dirname),
    ..opt
  };

  upload_data(client, data, opt).await
}

#[cfg(test)]
mod tests {
  use super::*;

  use crate::SkynetClientOptions;

  #[tokio::test]
  async fn test_upload_data() {
    let client = SkynetClient::default();
    let mut data = HashMap::new();
    data.insert("hello.txt".into(), (mime::TEXT_PLAIN, "hello world".into()));
    let res = upload_data(&client, data, UploadOptions::default()).await;
    println!("{:?}", res);
    assert!(res.is_ok());
    let skylink = res.unwrap();
    assert!(skylink.starts_with(URI_SKYNET_PREFIX));
  }

  #[tokio::test]
  async fn test_upload_file() {
    let client = SkynetClient::default();
    fs::write("tmp.txt", "hello world").unwrap();
    let res = upload_file(&client, &Path::new("tmp.txt"), UploadOptions::default()).await;
    fs::remove_file("tmp.txt").unwrap();
    println!("{:?}", res);
    assert!(res.is_ok());
    let skylink = res.unwrap();
    assert!(skylink.starts_with(URI_SKYNET_PREFIX));
  }

  #[tokio::test]
  async fn test_upload_file_tus_anon() {
    let client = SkynetClient::default();
    // generate 50MB file to trigger TUS upload
    fs::write("tmp.txt", (0..USE_TUS_THRESHOLD_BYTES+10000).map(|_| "X").collect::<String>()).unwrap();
    let path = Path::new("tmp.txt");
    let res = upload_data_tus(&client, path, UploadOptions::default()).await;
    fs::remove_file("tmp.txt").unwrap();
    println!("skylink: {:?}", &res);
    assert!(res.is_ok());
    let skylink = res.unwrap();
  }

  // make sure to set the SKYNET_API_KEY env var for this test
  #[tokio::test]
  async fn test_upload_file_tus_auth() {
    // generate 50MB file to trigger TUS upload
    let path = Path::new("tmp.txt");
    fs::write(path, (0..USE_TUS_THRESHOLD_BYTES+10000).map(|_| "X").collect::<String>()).unwrap();

    let client = SkynetClient::new("https://skynetfree.net", SkynetClientOptions {
      api_key: Some(std::env::var("SKYNET_API_KEY").unwrap()),
      custom_user_agent: None
    });

    let res = upload_data_tus(&client, path, UploadOptions::default()).await;
    fs::remove_file("tmp.txt").unwrap();
    println!("skylink: {:?}", &res);
    assert!(res.is_ok());
    let skylink = res.unwrap();
  }

  // make sure to set the SKYNET_API_KEY env var for this test
  #[tokio::test]
  async fn test_upload_file_tus_auth_large() {
    // generate 50MB file to trigger TUS upload
    let path = Path::new("tmp.txt");
    fs::write(path, (0..(700 * 1024 * 1024)).map(|_| "X").collect::<String>()).unwrap();

    let client = SkynetClient::new("https://skynetfree.net", SkynetClientOptions {
      api_key: Some(std::env::var("SKYNET_API_KEY").unwrap()),
      custom_user_agent: None
    });

    let res = upload_data_tus(&client, path, UploadOptions::default()).await;
    fs::remove_file("tmp.txt").unwrap();
    println!("skylink: {:?}", &res);
    assert!(res.is_ok());
    let skylink = res.unwrap();
  }

  #[tokio::test]
  async fn test_upload_directory() {
    let client = SkynetClient::default();
    fs::create_dir("tmpdir").unwrap();
    fs::write("tmpdir/1.txt", "hello 1").unwrap();
    fs::write("tmpdir/2.txt", "hello 2").unwrap();
    let res = upload_directory(&client, &Path::new("tmpdir"), UploadOptions::default()).await;
    fs::remove_dir_all("tmpdir").unwrap();
    println!("{:?}", res);
    assert!(res.is_ok());
    let skylink = res.unwrap();
    assert!(skylink.starts_with(URI_SKYNET_PREFIX));
  }
}
