use crate::util::DEFAULT_PORTAL_URL;

#[derive(Debug)]
pub struct SkynetClientOptions {
  pub api_key: Option<String>,
  pub custom_user_agent: Option<String>,
}

impl Default for SkynetClientOptions {
  fn default() -> Self {
    Self {
      api_key: None,
      custom_user_agent: None,
    }
  }
}

#[derive(Debug)]
pub struct SkynetClient {
  portal_url: String,
  options: SkynetClientOptions,
}

impl SkynetClient {
  pub fn new(portal_url: &str, opt: SkynetClientOptions) -> Self {
    Self {
      portal_url: portal_url.to_string(),
      options: opt,
    }
  }

  pub fn get_portal_url(&self) -> &str {
    self.portal_url.as_str()
  }
}

impl Default for SkynetClient {
  fn default() -> Self {
    Self::new(DEFAULT_PORTAL_URL, SkynetClientOptions::default())
  }
}
