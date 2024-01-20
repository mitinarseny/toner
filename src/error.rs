use thiserror::Error as ThisError;

pub type Result<T, E = Error> = core::result::Result<T, E>;

#[derive(Debug, ThisError)]
pub enum Error {
    #[error("too long")]
    TooLong,
    #[error("too many references")]
    TooManyReferences,
    #[error("value requires more space")]
    TooShort,
    #[error("more data left")]
    MoreLeft,
    #[error("no more data left")]
    NoMoreLeft,
}
