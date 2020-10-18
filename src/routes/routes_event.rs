use crate::connection_pool::PgsqlConn;
use crate::routes::RouteResult;
use crate::event::{Event, EventPlain, ToPlain};
use crate::database_helpers::{FromRow, get_cell_from_row};
use rocket_contrib::json::Json;
use crate::database_error::{DatabaseErrorKind, DatabaseError};
use chrono::{NaiveDate, NaiveTime};
use std::str::FromStr;
use postgres::types::ToSql;
use std::ops::Add;


fn get_event_by_id(db: &mut PgsqlConn, calendar_id: i32, event_id: i32) -> Result<Option<Event>, DatabaseError>
{
    let query = "SELECT * FROM events WHERE calendar_id = $1 AND id = $2";

    let rows = db.query(query, &[&calendar_id, &event_id])?;

    if let Some(row) = rows.get(0)
    {
        Ok(
            Some(Event::from_row(row)?)
        )
    }
    else
    {
        Ok(None)
    }
}


#[get("/calendars/<calendar_id>/events/<event_id>")]
pub fn get_event(mut db: PgsqlConn, calendar_id: i32, event_id: i32) -> RouteResult<EventPlain>
{
    get_event_by_id(&mut db, calendar_id, event_id)
        .map(|opt|
            opt.map(|event| event.into_plain())
        )
        .into()
}

#[post("/calendars/<calendar_id>/events", data = "<event>")]
pub fn insert_event(mut db: PgsqlConn, calendar_id: i32, event: Json<EventPlain>) -> RouteResult<EventPlain>
{
    if !event.validate_non_patch() || event.id.is_some()
    {
        return RouteResult::BadRequest(None);
    }

    let query = "INSERT INTO events
    (
        parent_event_id,
        start_date, start_time, end_date, end_time, rrule, exdates,
        rdates, calendar_id
    )

    VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
    RETURNING *;";

    let rows = db.query(query, &[
        &event.parent_id,
        &event.start_date,
        &event.start_time,
        &event.end_date,
        &event.end_time,
        &event.recurrence.as_ref().map(|r| &r.rrule),
        &event.recurrence.as_ref().map(|r| &r.exdates),
        &event.recurrence.as_ref().map(|r| &r.rdates),
        &calendar_id,
    ])?;

    if let Some(row) = rows.get(0)
    {
        RouteResult::Created(
            Event::from_row(row)?.into_plain(),
            //TODO: prepend host to url.
            format!("/api/calendars/{}/events/{}", calendar_id, get_cell_from_row::<i32>(row, "id")?)
        )
    }
    else
    {
        RouteResult::InternalError(Box::new(DatabaseError::from(DatabaseErrorKind::ReturningIsEmpty)))
    }
}

#[put("/calendars/<calendar_id>/events/<event_id>", data = "<event_data>")]
pub fn update_event(mut db: PgsqlConn, calendar_id: i32, event_id: i32, event_data: Json<EventPlain>) -> RouteResult<()>
{
    let mut query = "UPDATE events SET ".to_owned();


    let fields: Vec<(&str, Option<&(dyn ToSql + Sync)>)> = vec![
        ("start_date",  event_data.start_date   .as_ref()                       .map::<&(dyn ToSql + Sync), _>(|x| &*x)),
        ("end_date",    event_data.end_date     .as_ref()                       .map::<&(dyn ToSql + Sync), _>(|x| &*x)),
        ("start_time",  event_data.start_time   .as_ref()                       .map::<&(dyn ToSql + Sync), _>(|x| &*x)),
        ("end_time",    event_data.end_time     .as_ref()                       .map::<&(dyn ToSql + Sync), _>(|x| &*x)),
        ("rrule",       event_data.recurrence   .as_ref().and_then(|x| x.rrule      .as_ref().map::<&(dyn ToSql + Sync), _>(|x| &*x))),
        ("exdates",     event_data.recurrence   .as_ref().and_then(|x| x.exdates    .as_ref().map::<&(dyn ToSql + Sync), _>(|x| &*x))),
        ("rdates",      event_data.recurrence   .as_ref().and_then(|x| x.rdates     .as_ref().map::<&(dyn ToSql + Sync), _>(|x| &*x))),
    ];

    let mut param_counter = 0;

    // Iterate through each field and discard those that
    // have a value of None. For each of the remaining ones
    // we append `"field_name = $i,"` to the query, where
    // `field_name` is the name of the field and `i` is the
    // index+3 of the field.
    // (+3 because $1 is the calendar id and $2 is the event's id)
    let mut params: Vec<&(dyn ToSql + Sync)> = fields
        .into_iter()
        .filter_map::<&(dyn ToSql + Sync), _>(
            |(field, value)|
            {
                if value.is_some()
                {
                    param_counter += 1;
                    query = format!("{} {} = ${},", query, field, param_counter + 2);
                }
                value
            }
        )
        .collect();

    /// Remove the last comma ',' from the query. Panic if
    /// the character removed was not a comma.
    assert_eq!(query.remove(query.len() - 1), ',');

    query = query.add(" WHERE calendar_id = $1 AND id = $2 RETURNING *;");

    if params.len() > 0
    {
        params.insert(0, &calendar_id);
        params.insert(1, &event_id);


        db.execute(query.as_str(), &params)?;
    }

    RouteResult::Ok(())
}

#[get("/calendars/<calendar_id>/events/<event_id>/instances?<from>&<to>")]
pub fn get_instances(mut db: PgsqlConn, calendar_id: i32, event_id: i32, from: String, to: String) -> RouteResult<Vec<EventPlain>>
{
    if let Some(event) = get_event_by_id(&mut db, calendar_id, event_id)?
    {
        match event
        {
            Event::Recurring(event) => RouteResult::Ok(

                event
                    .generate_instances(NaiveDate::from_str(&from)?, NaiveDate::from_str(&to)?)?
                    .into_iter()
                    .map(|e| e.into_plain())
                    .collect()

            ),
            Event::Single(_) => RouteResult::NotFound,
        }
    }
    else
    {
        RouteResult::NotFound
    }
}