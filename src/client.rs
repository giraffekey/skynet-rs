use crate::util::DEFAULT_PORTAL_URL;

pub struct SkynetClientOptions {
  api_key: Option<String>,
  custom_user_agent: Option<String>,
}

impl SkynetClientOptions {
  pub fn new(api_key: Option<&str>, custom_user_agent: Option<&str>) -> Self {
  	Self {
      api_key: if let Some(api_key) = api_key {
        Some(api_key.to_string())
      } else {
        None
      },
      custom_user_agent: if let Some(custom_user_agent) = custom_user_agent {
        Some(custom_user_agent.to_string())
      } else {
        None
      },
    }
  }
}


impl Default for SkynetClientOptions {
  fn default() -> Self {
    Self::new(None, None)
  }
}

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
}

impl Default for SkynetClient {
  fn default() -> Self {
    Self::new(DEFAULT_PORTAL_URL, SkynetClientOptions::default())
  }
}
