use itertools::Itertools;
use thiserror::Error as ThisError;

pub type Result<T, E = Error> = core::result::Result<T, E>;

#[derive(Debug, ThisError)]
#[error(".{}: {reason}", .backtrace.iter().rev().join("."))]
pub struct Error {
    backtrace: Vec<usize>,
    #[source]
    reason: ErrorReason,
}

impl Error {
    pub fn with_nth(mut self, n: usize) -> Self {
        self.backtrace.push(n);
        self
    }
}

impl From<ErrorReason> for Error {
    fn from(reason: ErrorReason) -> Self {
        Error {
            backtrace: [].into(),
            reason,
        }
    }
}

#[derive(Debug, ThisError)]
pub enum ErrorReason {
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
