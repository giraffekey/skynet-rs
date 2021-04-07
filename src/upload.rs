use crate::{SkynetClient, util::build_request, URI_SKYNET_PREFIX};
use std::{
  collections::HashMap,
  io::Write,
  path::Path,
  str,
};
use hyper::{body, Client, Request};
use hyper_tls::HttpsConnector;
use mime::Mime;
use serde::Deserialize;
use textnonce::TextNonce;

#[derive(Debug)]
pub enum UploadError {
  NoCustomDirname,
  FailedTextNonce(String),
  IoWrite(std::io::Error),
  FromUtf8,
  BodyParse,
  HttpRequest(hyper::Error),
  PortalResponse(String),
}

use UploadError::*;

pub type SkynetResult<T> = Result<T, UploadError>;

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
      portal_directory_file_fieldname: "file[]".to_string(),
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
  let https = HttpsConnector::new();
  let hyper = Client::builder().build::<_, hyper::Body>(https);
  let req = Request::builder().method("POST");

  let mut query = HashMap::new();

  let (fieldname, filename) =
    if data.len() == 1 {
      if opt.custom_dirname.is_some() {
        (opt.portal_directory_file_fieldname.clone(), "".to_string())
      } else {
        (opt.portal_file_fieldname.clone(), "".to_string())
      }
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
  let boundary = TextNonce::sized(68).map_err(|s| FailedTextNonce(s))?.into_string().into_bytes();

  for (filename, (mime, bytes)) in &data {
    let disposition = format!("form-data; name=\"{}\"; filename=\"{}\"", fieldname, filename);
    let headers = format!("Content-Disposition: {}\r\nContent-Type: {}\r\n", disposition, mime);

    body.write_all(b"--").map_err(|e| IoWrite(e))?;
    body.write_all(&boundary).map_err(|e| IoWrite(e))?;
    body.write_all(b"\r\n").map_err(|e| IoWrite(e))?;
    body.write_all(headers.as_bytes()).map_err(|e| IoWrite(e))?;
    body.write_all(b"\r\n").map_err(|e| IoWrite(e))?;
    body.write_all(bytes).map_err(|e| IoWrite(e))?;
    body.write_all(b"\r\n").map_err(|e| IoWrite(e))?;
  }

  body.write_all(b"--").map_err(|e| IoWrite(e))?;
  body.write_all(&boundary).map_err(|e| IoWrite(e))?;
  body.write_all(b"--\r\n").map_err(|e| IoWrite(e))?;

  let content_type = format!("multipart/form-data; boundary={}", str::from_utf8(&boundary).map_err(|_| FromUtf8)?);

  let req = build_request(client, req, opt, None, Some(content_type), query);

  let req = req.body(body.into()).map_err(|_| BodyParse)?;
  let res = hyper.request(req).await.map_err(|e| HttpRequest(e))?;
  let body = body::to_bytes(res.into_body()).await.map_err(|_| BodyParse)?;
  let body_str = str::from_utf8(&body).map_err(|_| FromUtf8)?;
  let res: UploadResponse = serde_json::from_str(body_str)
    .map_err(|_| PortalResponse(body_str.to_string()))?;

  let skylink = format!("{}{}", URI_SKYNET_PREFIX, res.skylink);

  Ok(skylink)
}

pub async fn upload_file(
  client: &SkynetClient,
  path: &Path,
  opt: UploadOptions,
) -> SkynetResult<String> {
  upload_data(client, HashMap::new(), opt).await
}

pub async fn upload_directory(
  client: &SkynetClient,
  path: &Path,
  opt: UploadOptions,
) -> SkynetResult<String> {
  upload_data(client, HashMap::new(), opt).await
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
    assert!(res.is_ok());
    let skylink = res.unwrap();
    assert!(skylink.starts_with(URI_SKYNET_PREFIX));
  }
}
