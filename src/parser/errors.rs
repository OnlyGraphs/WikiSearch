use std::fmt;
use tonic::Status;

#[derive(Debug, Clone)]
pub enum QueryErrorKind {
    InvalidSyntax,
}

#[derive(Debug, Clone)]
pub struct QueryError {
    pub msg: String,
    pub pos: String,
    pub kind: QueryErrorKind,
}

impl fmt::Display for QueryError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{:?} Error in query at: ({:?}) - {:?}",
            self.kind,self.pos, self.msg
        )
    }
}

impl std::error::Error for QueryError {}

