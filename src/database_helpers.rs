use postgres::types::{FromSql, WasNull};
use postgres::Row;
use std::error::Error;
use crate::database_error::{DatabaseError, DatabaseErrorKind};

pub fn get_cell_from_row<'a, T: FromSql<'a>>(row: &'a Row, col: &str) -> Result<T, DatabaseError>
{
    match row.try_get(col)
    {
        Ok(value) => Ok(value),
        Err(error) =>
            match error.source()
            {
                //Return missing column error if the cause of the error is None,
                //otherwise we don't know what exactly caused the error, so wrap it
                //in a PostgresError
                None => Err(DatabaseErrorKind::MissingCol(col.to_owned()).into()),
                _ => Err(DatabaseErrorKind::PostgresError(error).into())
            }
    }
}

pub fn get_cell_from_row_with_default<'a, T: FromSql<'a>>(row: &'a Row, col: &str, default: T) -> Result<T, DatabaseError>
{
    //Honestly I was not very sure what I was doing when
    //writing this function. Sorry if it's ugly, bad practice,
    //or not idiomatic :/

    match row.try_get(col)
    {
        Ok(value) => Ok(value),
        Err(error) =>
            match error.source()
            {
                //Return missing column error if the cause of the error is None
                None => Err(DatabaseErrorKind::MissingCol(col.to_owned()).into()),
                Some(err) =>
                //Return the default value if the cause of the error is
                //a WasNull error
                    match err.downcast_ref::<WasNull>()
                    {
                        Some(_) => Ok(default),
                        None => Err(DatabaseErrorKind::PostgresError(error).into())
                    },
            }
    }
}