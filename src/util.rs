use std::collections::HashMap;
use http::uri::Authority;
use hyper::Uri;

pub const DEFAULT_PORTAL_URL: &str = "https://siasky.net";
pub const URI_SKYNET_PREFIX: &str = "sia://";

pub fn make_uri(
  portal_url: &str,
  path: String,
  api_key: Option<String>,
  extra_path: Option<String>,
  query: HashMap<String, String>,
) -> Uri {
  let parts: Vec<&str> = portal_url.split("://").collect();
  let scheme = parts[0];
  let host = parts[1];

  // let authority: Authority = if let Some(api_key) = api_key {
  //   format!("{}@{}", api_key, host).parse().unwrap()
  // } else {
  //   host.parse().unwrap()
  // };

  let authority: Authority = host.parse().unwrap();

  let extra_path = if let Some(extra_path) = extra_path {
    format!("/{}", extra_path)
  } else {
    "".to_string()
  };

  let query = if query.is_empty() {
    "".to_string()
  } else {
    let query = query
      .iter()
      .map(|(k, v)| format!("{}={}", k, v))
      .collect::<Vec<String>>()
      .join("&");
    format!("?{}", query)
  };

  let path_and_query = format!("{}{}{}", path, extra_path, query);

  Uri::builder()
    .scheme(scheme)
    .authority(authority)
    .path_and_query(path_and_query)
    .build()
    .unwrap()
}
