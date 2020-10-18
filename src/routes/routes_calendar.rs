use crate::connection_pool::PgsqlConn;
use crate::calendar::{Calendar};
use crate::routes::RouteResult;
use crate::database_helpers::FromRow;
use rocket_contrib::json::Json;
use crate::database_error::{DatabaseError, DatabaseErrorKind};

/// Gets a calendar by id from the database.
///
/// Response codes: 200, 404, 500
#[get("/calendars/<calendar_id>")]
pub fn get_calendar(mut db: PgsqlConn, calendar_id: i32) -> RouteResult<Calendar>
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

/// Inserts a calendar into the database and returns it.
///
/// Response codes: 201, 500
#[post("/calendars", data = "<calendar>")]
pub fn insert_calendar(mut db: PgsqlConn, calendar: Json<Calendar>) -> RouteResult<Calendar>
{
    let calendar = calendar.into_inner();

    if calendar.get_id() != -1
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