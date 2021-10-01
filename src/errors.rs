use thiserror::Error;

#[derive(Error, Debug)]
pub enum GeneralError {
    #[error("file format")]
    InvalidFormat(String),
    #[error("unknown error")]
    Unknown,
}
