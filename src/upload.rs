use crate::{SkynetClient, SkynetResult};
use std::{
  collections::HashMap,
  path::Path,
};
use http::Request;

pub struct UploadOptions {
  endpoint_path: String,
  api_key: Option<String>,
  custom_user_agent: Option<String>,
  portal_file_fieldname: String,
  portal_directory_file_fieldname: String,
  custom_filename: Option<String>,
  custom_dirname: Option<String>,
}

impl UploadOptions {
  pub fn new(
  	endpoint_path: &str,
  	api_key: Option<&str>,
  	custom_user_agent: Option<&str>,
  	portal_file_fieldname: &str,
  	portal_directory_file_fieldname: &str,
  	custom_filename: Option<String>,
  	custom_dirname: Option<String>,
  ) -> Self {
  	Self {
  	  endpoint_path: endpoint_path.to_string(),
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
      portal_file_fieldname: portal_file_fieldname.to_string(),
      portal_directory_file_fieldname: portal_directory_file_fieldname.to_string(),
      custom_filename: if let Some(custom_filename) = custom_filename {
        Some(custom_filename.to_string())
      } else {
        None
      },
      custom_dirname: if let Some(custom_dirname) = custom_dirname {
        Some(custom_dirname.to_string())
      } else {
        None
      },
  	}
  }
}

impl Default for UploadOptions {
  fn default() -> Self {
  	Self::new("/skynet/skyfile", None, None, "file", "file[]", None, None)
  }
}

pub fn upload_data(
  client: &SkynetClient,
  data: HashMap<String, Vec<u8>>,
  opt: UploadOptions,
) -> SkynetResult<String> {
  let skylink = "".to_string();

  Ok(skylink)
}

pub fn upload_file(client: &SkynetClient, path: &Path, opt: UploadOptions) -> SkynetResult<String> {
  upload_data(client, HashMap::new(), opt)
}

pub fn upload_directory(client: &SkynetClient, path: &Path, opt: UploadOptions) -> SkynetResult<String> {
  upload_data(client, HashMap::new(), opt)
}
