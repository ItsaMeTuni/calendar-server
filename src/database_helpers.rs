use postgres::types::{FromSql, WasNull, ToSql, Type, IsNull};
use postgres::Row;
use std::error::Error;
use crate::database_error::{DatabaseError, DatabaseErrorKind};
use uuid::Uuid;
use rocket::request::FromParam;
use rocket::http::RawStr;
use std::str::FromStr;
use postgres::types::private::BytesMut;
use serde::export::fmt::Display;
use serde::export::Formatter;
use rocket_okapi::request::OpenApiFromParam;
use rocket_okapi::gen::OpenApiGenerator;
use okapi::openapi3::{Parameter, ParameterValue};

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

pub trait FromRow
{
    type SelfType;

    fn from_row(row: &Row) -> Result<Self::SelfType, DatabaseError>;
}

#[derive(Debug)]
pub struct UuidParam(Uuid);
impl UuidParam
{
    pub fn into_inner(self) -> Uuid { self.0 }
}

impl Display for UuidParam
{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        self.0.fmt(f)
    }
}

impl FromParam<'_> for UuidParam
{
    type Error = uuid::Error;

    fn from_param(param: &RawStr) -> Result<Self, Self::Error>
    {
        Uuid::from_str(param.as_str())
            .map(|uuid| UuidParam(uuid))
    }
}

impl OpenApiFromParam<'_> for UuidParam
{
    fn path_parameter(gen: &mut OpenApiGenerator, name: String) -> rocket_okapi::Result<Parameter>
    {
        let schema = gen.json_schema::<Uuid>();
        Ok(Parameter {
            name,
            location: "path".to_owned(),
            description: None,
            required: true,
            deprecated: false,
            allow_empty_value: false,
            value: ParameterValue::Schema {
                style: None,
                explode: None,
                allow_reserved: false,
                schema,
                example: None,
                examples: None,
            },
            extensions: Default::default(),
        })
    }
}

impl ToSql for UuidParam
{
    fn to_sql(&self, ty: &Type, out: &mut BytesMut) -> Result<IsNull, Box<dyn Error + Sync + Send>> where
        Self: Sized {
        self.0.to_sql(ty, out)
    }

    fn accepts(ty: &Type) -> bool where
        Self: Sized {
        <Uuid as ToSql>::accepts(ty)
    }

    fn to_sql_checked(&self, ty: &Type, out: &mut BytesMut) -> Result<IsNull, Box<dyn Error + Sync + Send>> {
        self.0.to_sql_checked(ty, out)
    }
}