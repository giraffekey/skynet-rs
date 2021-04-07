pub const DEFAULT_PORTAL_URL: &str = "https://siasky.net";

pub const URI_SKYNET_PREFIX: &str = "sia://";

pub fn make_url(
  portal_url: &str,
  path: &str,
  extra_path: Option<&str>,
  query: Option<&str>,
) -> String {
  let extra_path = if let Some(extra_path) = extra_path {
    format!("/{}", extra_path)
  } else {
    "".to_string()
  };

  let query = if let Some(query) = query {
    format!("?{}", query)
  } else {
    "".to_string()
  };

  format!("{}/{}{}{}", portal_url, path, extra_path, query)
}
