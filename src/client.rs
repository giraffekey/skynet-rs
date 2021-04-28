use crate::{
  upload, download, encryption,
  UploadOptions,
  DownloadOptions, MetadataOptions, Metadata,
  Skykey, SkykeyOptions,
  SkynetResult,
  util::DEFAULT_PORTAL_URL,
};
use std::{collections::HashMap, path::Path};
use hyper::{client::HttpConnector, Client};
use hyper_tls::HttpsConnector;
use mime::Mime;

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
  pub http: Client<HttpsConnector<HttpConnector>>,
}

impl SkynetClient {
  pub fn new(portal_url: &str, opt: SkynetClientOptions) -> Self {
    let https = HttpsConnector::new();
    let http = Client::builder().build::<_, hyper::Body>(https);

    Self {
      portal_url: portal_url.to_string(),
      options: opt,
      http,
    }
  }

  pub fn get_portal_url(&self) -> &str {
    self.portal_url.as_str()
  }

  pub async fn upload_data(
    &self,
    data: HashMap<String, (Mime, Vec<u8>)>,
    opt: UploadOptions,
  ) -> SkynetResult<String> {
    upload::upload_data(self, data, opt).await
  }

  pub async fn upload_file<P: AsRef<Path>>(
    &self,
    path: P,
    opt: UploadOptions,
  ) -> SkynetResult<String> {
    upload::upload_file(self, path.as_ref(), opt).await
  }

  pub async fn upload_directory<P: AsRef<Path>>(
    &self,
    path: P,
    opt: UploadOptions,
  ) -> SkynetResult<String> {
    upload::upload_directory(self, path.as_ref(), opt).await
  }

  pub async fn download_data(
    &self,
    skylink: &str,
    opt: DownloadOptions,
  ) -> SkynetResult<Vec<u8>> {
    download::download_data(self, skylink, opt).await
  }

  pub async fn download_file<P: AsRef<Path>>(
    &self,
    path: P,
    skylink: &str,
    opt: DownloadOptions,
  ) -> SkynetResult<()> {
    download::download_file(self, path, skylink, opt).await
  }

  pub async fn get_metadata(
    &self,
    skylink: &str,
    opt: MetadataOptions,
  ) -> SkynetResult<Metadata> {
    download::get_metadata(self, skylink, opt).await
  }

  pub async fn add_skykey(
    &self,
    skykey: &str,
    opt: SkykeyOptions,
  ) -> SkynetResult<()> {
    encryption::add_skykey(self, skykey, opt).await
  }

  pub async fn create_skykey(
    &self,
    name: &str,
    skykey_type: &str,
    opt: SkykeyOptions,
  ) -> SkynetResult<Skykey> {
    encryption::create_skykey(self, name, skykey_type, opt).await
  }

  pub async fn get_skykey_by_name(
    &self,
    name: &str,
    opt: SkykeyOptions,
  ) -> SkynetResult<Skykey> {
    encryption::get_skykey_by_name(self, name, opt).await
  }

  pub async fn get_skykey_by_id(
    &self,
    id: &str,
    opt: SkykeyOptions,
  ) -> SkynetResult<Skykey> {
    encryption::get_skykey_by_id(self, id, opt).await
  }

  pub async fn get_skykeys(&self, opt: SkykeyOptions) -> SkynetResult<Vec<Skykey>> {
    encryption::get_skykeys(self, opt).await
  }
}

impl Default for SkynetClient {
  fn default() -> Self {
    Self::new(DEFAULT_PORTAL_URL, SkynetClientOptions::default())
  }
}
