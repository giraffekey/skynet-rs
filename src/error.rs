#[derive(Debug)]
pub enum SkynetError {
  NoCustomDirname,
  NotFile,
  NotDirectory,
  TextNonceError(String),
  WriteError(std::io::Error),
  FileError(std::io::Error),
  TUSError(tus_async_client::Error),
  HttpError(http::Error),
  HyperError(hyper::Error),
  ReqwestError(reqwest::Error),
  Utf8Error(std::str::Utf8Error),
  PortalResponse(String),
  InvalidSignature,
}

pub type SkynetResult<T> = Result<T, SkynetError>;
