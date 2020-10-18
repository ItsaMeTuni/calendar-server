use std::error::Error;
use std::backtrace::Backtrace;

#[derive(Error, Debug)]
#[error("{kind:?}")]
pub struct DatabaseError
{
    kind: DatabaseErrorKind,
    backtrace: Backtrace,
}

#[derive(Debug)]
pub enum DatabaseErrorKind
{
    FailedConstraint(String),
    MissingCol(String),
    UnexpectedNull(String),
    PostgresError(postgres::Error),
    ExpectedRow(i32),

    /// A query with the RETURNING clause returned 0 rows.
    ReturningIsEmpty,

    Other(Box<dyn Error>),
}


impl From<DatabaseErrorKind> for DatabaseError
{
    fn from(kind: DatabaseErrorKind) -> Self
    {
        DatabaseError {
            kind,
            backtrace: Backtrace::capture(),
        }
    }
}

impl From<postgres::Error> for DatabaseError
{
    fn from(e: postgres::Error) -> Self
    {
        DatabaseError {
            kind: DatabaseErrorKind::PostgresError(e),
            backtrace: Backtrace::capture(),
        }
    }
}