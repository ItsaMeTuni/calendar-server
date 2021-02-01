use crate::connection_pool::PgsqlConn;
use crate::calendar::{Calendar};
use crate::routes::RouteResult;
use crate::database_helpers::{FromRow, UuidParam};
use rocket_contrib::json::Json;
use crate::database_error::{DatabaseError, DatabaseErrorKind};
use crate::routes::common_query_params::CommonQueryParams;
use crate::authentication::auth_guard::ApiKey;

/// Gets a calendar by id from the database.
///
/// Response codes: 200, 404, 500
#[openapi]
#[get("/calendars/<calendar_id>")]
pub fn get_calendar(mut db: PgsqlConn, _api_key: ApiKey, calendar_id: UuidParam) -> RouteResult<Calendar>
{
    let query = "SELECT * FROM calendars WHERE id = $1;";

    let query = db.query(query, &[&calendar_id])?;

    if let Some(row) = query.get(0)
    {
        RouteResult::Ok(
            Calendar::from_row(row)?
        )
    }
    else
    {
        RouteResult::NotFound
    }
}

/// Lists all calendars in the database.
///
/// Response codes: 200, 500
#[openapi]
#[get("/calendars")]
pub fn list_calendars(mut db: PgsqlConn, _api_key: ApiKey, shared_params: CommonQueryParams) -> RouteResult<Vec<Calendar>>
{
    let query = "SELECT * FROM calendars OFFSET $1 LIMIT $2;";

    let rows = db.query(query, &[
        &shared_params.offset(),
        &shared_params.page_size(),
    ])?;

    RouteResult::Ok(
        rows.into_iter()
            .map(|row| Calendar::from_row(&row))
            .collect::<Result<Vec<_>, _>>()?
    )
}


/// Inserts a calendar into the database and returns it.
///
/// Response codes: 201, 500
#[openapi]
#[post("/calendars", data = "<calendar>")]
pub fn insert_calendar(mut db: PgsqlConn, _api_key: ApiKey, calendar: Json<Calendar>) -> RouteResult<Calendar>
{
    let calendar = calendar.into_inner();

    if !calendar.get_id().is_nil()
    {
        RouteResult::BadRequest(None)
    }
    else
    {
        let query = "INSERT INTO calendars DEFAULT VALUES RETURNING *";

        let rows = db.query(query, &[])?;

        if let Some(row) = rows.get(0)
        {
            RouteResult::Ok(Calendar::from_row(row)?)
        }
        else
        {
            RouteResult::InternalError(Box::<DatabaseError>::new(DatabaseErrorKind::ReturningIsEmpty.into()))
        }
    }
}