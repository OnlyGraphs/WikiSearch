use std::fmt;
use tonic::Status;

#[derive(Debug, Clone)]
pub enum IndexErrorKind {
    Database,
    PoisonedThread,
    InvalidIndexState,
    InvalidOperation,
    BuildFailed,
    GRPCBadStatus,
    LogicError,
    Error,
}

#[derive(Debug, Clone)]
pub struct IndexError {
    pub msg: String,
    pub kind: IndexErrorKind,
}


impl fmt::Display for IndexError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Error in building/accessing index ({:?}): {:?}",
            self.kind, self.msg
        )
    }
}

impl std::error::Error for IndexError {}

impl From<sqlx::Error> for IndexError {
    fn from(e: sqlx::Error) -> Self {
        IndexError {
            msg: e.to_string(),
            kind: IndexErrorKind::Database,
        }
    }
}

impl From<IndexError> for Status {
    fn from(e: IndexError) -> Self {
        Status::new(tonic::Code::Unknown, e.to_string())
    }
}

impl From<std::io::Error> for IndexError {
    fn from(e: std::io::Error) -> Self {
        IndexError {
            msg: e.to_string(),
            kind: IndexErrorKind::Error,
        }
    }
}

impl From<IndexError> for std::io::Error {
    fn from(e: IndexError) -> Self {
        std::io::Error::new(std::io::ErrorKind::Other, e)
    }
}

