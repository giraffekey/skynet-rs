use crate::{SkynetClient, SkynetError::*, SkynetResult, util::make_uri};
use std::{collections::HashMap, str};
use hyper::{body, Body, Request};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Skykey {
  skykey: String,
  name: String,
  id: String,
  r#type: String,
}

#[derive(Debug)]
pub struct SkykeyOptions {
  pub endpoint_path: Option<String>,
  pub api_key: Option<String>,
  pub custom_user_agent: Option<String>,
}

impl Default for SkykeyOptions {
  fn default() -> Self {
    Self {
      endpoint_path: None,
      api_key: None,
      custom_user_agent: None,
    }
  }
}

pub async fn add_skykey(
  client: &SkynetClient,
  skykey: &str,
  opt: SkykeyOptions,
) -> SkynetResult<()> {
  let req = Request::builder().method("POST");

  let mut query = HashMap::new();

  let endpoint_path = if let Some(endpoint_path) = opt.endpoint_path {
    endpoint_path
  } else {
    "/skynet/addskykey".into()
  };

  query.insert("skykey".into(), skykey.into());

  let uri = make_uri(
    client.get_portal_url(),
    endpoint_path,
    opt.api_key,
    None,
    query);

  let mut req = req.uri(uri);

  if let Some(custom_user_agent) = opt.custom_user_agent {
    req = req.header("User-Agent", custom_user_agent);
  }

  let req = req.body(Body::from("")).map_err(HttpError)?;
  client.http.request(req).await.map_err(HyperError)?;

  Ok(())
}

pub async fn create_skykey(
  client: &SkynetClient,
  name: &str,
  skykey_type: &str,
  opt: SkykeyOptions,
) -> SkynetResult<Skykey> {
  let req = Request::builder().method("POST");

  let mut query = HashMap::new();

  let endpoint_path = if let Some(endpoint_path) = opt.endpoint_path {
    endpoint_path
  } else {
    "/skynet/createskykey".into()
  };

  query.insert("name".into(), name.into());
  query.insert("type".into(), skykey_type.into());

  let uri = make_uri(
    client.get_portal_url(),
    endpoint_path,
    opt.api_key,
    None,
    query);

  let mut req = req.uri(uri);

  if let Some(custom_user_agent) = opt.custom_user_agent {
    req = req.header("User-Agent", custom_user_agent);
  }

  let req = req.body(Body::from("")).map_err(HttpError)?;
  let res = client.http.request(req).await.map_err(HyperError)?;
  let body = body::to_bytes(res.into_body()).await.map_err(HyperError)?;
  let body_str = str::from_utf8(&body).map_err(Utf8Error)?;
  let skykey: Skykey = serde_json::from_str(body_str)
    .map_err(|_| PortalResponse(body_str.to_string()))?;

  Ok(skykey)
}

pub async fn get_skykey_by_name(
  client: &SkynetClient,
  name: &str,
  opt: SkykeyOptions,
) -> SkynetResult<Skykey> {
  let req = Request::builder().method("GET");

  let mut query = HashMap::new();

  let endpoint_path = if let Some(endpoint_path) = opt.endpoint_path {
    endpoint_path
  } else {
    "/skynet/skykey".into()
  };

  query.insert("name".into(), name.into());

  let uri = make_uri(
    client.get_portal_url(),
    endpoint_path,
    opt.api_key,
    None,
    query);

  let mut req = req.uri(uri);

  if let Some(custom_user_agent) = opt.custom_user_agent {
    req = req.header("User-Agent", custom_user_agent);
  }

  let req = req.body(Body::from("")).map_err(HttpError)?;
  let res = client.http.request(req).await.map_err(HyperError)?;
  let body = body::to_bytes(res.into_body()).await.map_err(HyperError)?;
  let body_str = str::from_utf8(&body).map_err(Utf8Error)?;
  let skykey: Skykey = serde_json::from_str(body_str)
    .map_err(|_| PortalResponse(body_str.to_string()))?;

  Ok(skykey)
}

pub async fn get_skykey_by_id(
  client: &SkynetClient,
  id: &str,
  opt: SkykeyOptions,
) -> SkynetResult<Skykey> {
  let req = Request::builder().method("GET");

  let mut query = HashMap::new();

  let endpoint_path = if let Some(endpoint_path) = opt.endpoint_path {
    endpoint_path
  } else {
    "/skynet/skykey".into()
  };

  query.insert("id".into(), id.into());

  let uri = make_uri(
    client.get_portal_url(),
    endpoint_path,
    opt.api_key,
    None,
    query);

  let mut req = req.uri(uri);

  if let Some(custom_user_agent) = opt.custom_user_agent {
    req = req.header("User-Agent", custom_user_agent);
  }

  let req = req.body(Body::from("")).map_err(HttpError)?;
  let res = client.http.request(req).await.map_err(HyperError)?;
  let body = body::to_bytes(res.into_body()).await.map_err(HyperError)?;
  let body_str = str::from_utf8(&body).map_err(Utf8Error)?;
  let skykey: Skykey = serde_json::from_str(body_str)
    .map_err(|_| PortalResponse(body_str.to_string()))?;

  Ok(skykey)
}

pub async fn get_skykeys(
  client: &SkynetClient,
  opt: SkykeyOptions,
) -> SkynetResult<Vec<Skykey>> {
  let req = Request::builder().method("GET");

  let query = HashMap::new();

  let endpoint_path = if let Some(endpoint_path) = opt.endpoint_path {
    endpoint_path
  } else {
    "/skynet/skykeys".into()
  };

  let uri = make_uri(
    client.get_portal_url(),
    endpoint_path,
    opt.api_key,
    None,
    query);

  let mut req = req.uri(uri);

  if let Some(custom_user_agent) = opt.custom_user_agent {
    req = req.header("User-Agent", custom_user_agent);
  }

  let req = req.body(Body::from("")).map_err(HttpError)?;
  let res = client.http.request(req).await.map_err(HyperError)?;
  let body = body::to_bytes(res.into_body()).await.map_err(HyperError)?;
  let body_str = str::from_utf8(&body).map_err(Utf8Error)?;
  let skykey: Vec<Skykey> = serde_json::from_str(body_str)
    .map_err(|_| PortalResponse(body_str.to_string()))?;

  Ok(skykey)
}
