use r2d2_postgres::r2d2::{Pool, PooledConnection};
use r2d2_postgres::PostgresConnectionManager;
use postgres::NoTls;
use std::ops::{Deref, DerefMut};
use rocket::request::{FromRequest, Outcome};
use crate::database_error::{DatabaseError, DatabaseErrorKind};
use rocket::http::Status;
use rocket::{Request, State};

pub struct PgsqlPool
{
    pool: Pool<PostgresConnectionManager<NoTls>>,
}

impl PgsqlPool
{
    pub fn new(settings: &str) -> PgsqlPool
    {
        let manager = PostgresConnectionManager::new(
            settings.parse().unwrap(),
            NoTls,
        );

        let pool = Pool::new(manager).unwrap();

        PgsqlPool { pool }
    }

    pub fn get_conn(&self) -> Result<PgsqlConn, r2d2_postgres::r2d2::Error>
    {
        Ok( PgsqlConn { conn: self.pool.get()? } )
    }
}

pub struct PgsqlConn
{
    conn: PooledConnection<PostgresConnectionManager<NoTls>>,
}

impl Deref for PgsqlConn
{
    type Target = PooledConnection<PostgresConnectionManager<NoTls>>;

    fn deref(&self) -> &Self::Target
    {
        &self.conn
    }
}

impl DerefMut for PgsqlConn
{
    fn deref_mut(&mut self) -> &mut Self::Target
    {
        &mut self.conn
    }
}

#[derive(Error, Debug)]
#[error("Failed to get a connection from the pool.")]
struct PoolGetFail {}

impl<'a, 'r> FromRequest<'a, 'r> for PgsqlConn
{
    type Error = DatabaseError;

    fn from_request(request: &Request) -> Outcome<Self, Self::Error>
    {
        let pool: State<PgsqlPool> = match request.guard::<State<PgsqlPool>>()
        {
            Outcome::Success(p) => p,
            _ => return Outcome::Failure((Status::InternalServerError, DatabaseErrorKind::Other(Box::new(PoolGetFail {})).into())),
        };

        let conn = match pool.pool.get()
        {
            Ok(c) => c,
            Err(_) => return Outcome::Failure((Status::InternalServerError, DatabaseErrorKind::Other(Box::new(PoolGetFail {})).into())),
        };

        Outcome::Success(PgsqlConn { conn })
    }
}