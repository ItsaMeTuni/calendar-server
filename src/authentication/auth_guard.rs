use rocket::request::{FromRequest, Outcome};
use rocket::Request;
use rocket::http::Status;
use crate::connection_pool::PgsqlConn;
use crate::database_helpers::{get_cell_from_row, UuidParam};
use rocket::outcome::IntoOutcome;
use uuid::Uuid;
use std::str::FromStr;

pub struct InvalidScopeError;

pub struct ApiKey
{
    key: Uuid,

    // TODO: use a bitmask here to reduce allocations (?)
    scopes: Vec<String>,
}

impl<'a, 'r> FromRequest<'a, 'r> for ApiKey
{
    type Error = ();

    fn from_request(request: &'a Request<'r>) -> Outcome<Self, Self::Error> {

        // TODO: improve error handling here, this is horrendous.

        // get Authorization header
        let api_key = request.headers()
            .get_one("Authorization")
            .and_then(|x| Uuid::from_str(x).ok())
            .into_outcome((Status::Unauthorized, ()))?;

        // get api key scopes from db
        let mut db = request.guard::<PgsqlConn>().unwrap();
        let query = "SELECT scopes FROM api_keys WHERE api_key = $1;";
        let rows = db.query(query, &[&api_key]).unwrap();

        if let Some(row) = rows.get(0)
        {
            Outcome::Success(
                ApiKey {
                    key: api_key,
                    scopes: get_cell_from_row( & row, "scopes").ok().into_outcome((Status::InternalServerError, ()))?
                }
            )
        }
        else
        {
            Outcome::Failure((Status::Unauthorized, ()))
        }
    }
}