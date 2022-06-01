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

#[derive(Debug)]
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

  // let skylink = format!("{}{}", URI_SKYNET_PREFIX, res.skylink);

  Ok(res.skylink)
}

pub async fn upload_file(
  client: &SkynetClient,
  path: &Path,
  opt: UploadOptions,
) -> SkynetResult<String> {
  if !path.is_file() {
    return Err(NotFile);
  }

  let filename = path.file_name().unwrap().to_str().unwrap().to_string();
  let mime = if let Some(mime) = mime_guess::from_path(path).first() {
    mime
  } else {
    mime::APPLICATION_OCTET_STREAM
  };
  let bytes = fs::read(path).map_err(FileError)?;

  let mut data = HashMap::new();
  data.insert(filename, (mime, bytes));

  upload_data(client, data, opt).await
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
