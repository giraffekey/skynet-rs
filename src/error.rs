pub enum SkynetError {
  Error,
}

pub type SkynetResult<T> = Result<T, SkynetError>;
