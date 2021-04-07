#[derive(Debug)]
pub enum SkynetError {
  NoCustomDirname,
  NotFile,
  NotDirectory,
  TextNonceError(String),
  WriteError(std::io::Error),
  FileError(std::io::Error),
  HttpError(http::Error),
  HyperError(hyper::Error),
  Utf8Error(std::str::Utf8Error),
  PortalResponse(String),
}

pub type SkynetResult<T> = Result<T, SkynetError>;
