use crate::connection_pool::PgsqlConn;
use crate::database_error::{DatabaseError, DatabaseErrorKind};
use postgres::Row;
use crate::database_helpers::get_cell_from_row;
use super::event::{EVENT_FIELDS, Event};
use crate::calendar::event::EventPlain;

pub const CALENDAR_FIELDS: &str = "id, tenant_id";

#[derive(Debug)]
pub struct Calendar
{
    id: i32,
}


/// This happens if you call Calendar::insert_event and the
/// `event` param was Some.
#[derive(Error, Debug)]
#[error("Tried to insert an event but the EventPlain's id was Some.")]
struct InsertCannotHaveId(EventPlain);

impl Calendar
{
    pub fn new(id: i32) -> Calendar
    {
        Calendar {
            id
        }
    }

    pub fn get_id(&self) -> i32 { self.id }
}

pub fn get_event_by_id(db: &mut PgsqlConn, tenant_id: i32, calendar_id: i32, event_id: i32) -> Result<Option<Event>, DatabaseError>
{
    let query = "
            SELECT id, parent_event_id, start_date,
                start_time, end_date, end_time, rrule,
                exdates, rdates
            FROM events
            INNER JOIN calendars ON events.calendar_id = calendars.id
            WHERE calendars.tenant_id = $1 AND calendar_id = $2 AND events.id = $3;
         ";

    let rows = db.query(query, &[&tenant_id, &calendar_id, &event_id])?;

    rows.get(0)
        .map(|row| Event::from_row(row))
        .transpose()

}

pub fn list_events(db: &mut PgsqlConn, tenant_id: i32, calendar_id: i32) -> Result<Vec<Event>, DatabaseError>
{
    let query = "
            SELECT id, parent_event_id, start_date,
                start_time, end_date, end_time, rrule,
                exdates, rdates
            FROM events
            INNER JOIN calendars ON events.calendar_id = calendars.id
            WHERE calendars.tenant_id = $1 AND events.calendar_id = $2;
         ";

    let rows = db.query(query, &[&tenant_id, &calendar_id])?;

    rows
        .into_iter()
        .map(|row| Event::from_row(&row))
        .collect()
}

pub fn insert_event(db: &mut PgsqlConn, tenant_id: i32, calendar_id: i32, event: EventPlain) -> Result<i32, DatabaseError>
{
    let query = "
            INSERT INTO events (calendar_id, parent_event_id, start_date,
                start_time, end_date, end_time, rrule, exdates,
                rdates)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING id;
        ";

    if event.id.is_some()
    {
        return Err(DatabaseErrorKind::Other(Box::new(InsertCannotHaveId(event))).into());
    }

    let result = db.query(query, &[
        &calendar_id,
        &event.parent_id,
        &event.start_date,
        &event.start_time,
        &event.end_date,
        &event.end_time,
        &event.recurrence.as_ref().map(|x| &x.rrule),
        &event.recurrence.as_ref().map(|x| &x.exdates),
        &event.recurrence.as_ref().map(|x| &x.rdates),
    ])?;

    if let Some(row) = result.get(0)
    {
        Ok(get_cell_from_row(&row, "id")?)
    }
    else
    {
        Err(DatabaseErrorKind::ExpectedRow(0).into())
    }
}