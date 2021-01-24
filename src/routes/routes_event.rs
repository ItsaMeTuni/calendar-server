use crate::connection_pool::PgsqlConn;
use crate::routes::RouteResult;
use crate::event::{Event, EventPlain, ToPlain};
use crate::database_helpers::{FromRow, get_cell_from_row, UuidParam};
use rocket_contrib::json::Json;
use crate::database_error::{DatabaseErrorKind, DatabaseError};
use std::ops::Add;
use rocket::request::{FromFormValue};
use rocket::http::RawStr;
use chrono::{NaiveDateTime, NaiveDate, NaiveTime};
use postgres::types::{ToSql};


use std::fmt::Debug;
use std::str::FromStr;
use crate::routes::common_query_params::CommonQueryParams;


/// Store a NaiveDate, NaiveTime or NaiveDateTime without knowing
/// exactly which one it is. Generally used as route query parameters
/// that accept more than one of these types.
///
/// ToSql is purposefully not implemented for this type, since query parameters
/// can only have one type and NaiveDateOrTime can map to three different
/// SQL types (TIMESTAMP, DATE, and TIME).
#[derive(Debug)]
pub enum NaiveDateOrTime
{
    Date(NaiveDate),
    Time(NaiveTime),
    DateTime(NaiveDateTime)
}

impl NaiveDateOrTime
{
    pub fn as_naive_date(&self) -> Option<&NaiveDate>
    {
        match self
        {
            NaiveDateOrTime::Date(d) => Some(d),
            NaiveDateOrTime::Time(_) => None,
            NaiveDateOrTime::DateTime(_) => None,
        }
    }

    pub fn as_naive_time(&self) -> Option<&NaiveTime>
    {
        match self
        {
            NaiveDateOrTime::Date(_) => None,
            NaiveDateOrTime::Time(t) => Some(t),
            NaiveDateOrTime::DateTime(_) => None,
        }
    }

    pub fn as_naive_date_time(&self) -> Option<&NaiveDateTime>
    {
        match self
        {
            NaiveDateOrTime::Date(_) => None,
            NaiveDateOrTime::Time(_) => None,
            NaiveDateOrTime::DateTime(dt) => Some(dt),
        }
    }
}

impl FromStr for NaiveDateOrTime
{
    type Err = chrono::ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err>
    {
        NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M")
            .map(|dt| NaiveDateOrTime::DateTime(dt))

            .or_else(|_|
                NaiveDate::parse_from_str(s, "%Y-%m-%d")
                    .map(|d| NaiveDateOrTime::Date(d))
            )

            .or_else(|_|
                NaiveTime::parse_from_str(s, "%H:%M")
                    .map(|t| NaiveDateOrTime::Time(t))
            )
    }
}

impl<'v> FromFormValue<'v> for NaiveDateOrTime
{
    type Error = chrono::ParseError;

    fn from_form_value(form_value: &'v RawStr) -> Result<Self, Self::Error>
    {
        NaiveDateOrTime::from_str(form_value.as_str())
    }
}

pub struct NaiveDateParam(NaiveDate);
impl NaiveDateParam
{
    pub fn into_inner(self) -> NaiveDate
    {
        self.0
    }
}

impl<'v> FromFormValue<'v> for NaiveDateParam
{
    type Error = chrono::ParseError;

    fn from_form_value(param: &'v RawStr) -> Result<Self, Self::Error> {
        NaiveDate::from_str(param.as_str())
            .map(|x| NaiveDateParam(x))
    }
}




fn get_event_by_id(db: &mut PgsqlConn, calendar_id: UuidParam, event_id: UuidParam) -> Result<Option<Event>, DatabaseError>
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
pub fn get_event(mut db: PgsqlConn, calendar_id: UuidParam, event_id: UuidParam) -> RouteResult<EventPlain>
{
    get_event_by_id(&mut db, calendar_id, event_id)
        .map(|opt|
            opt.map(|event| event.into_plain())
        )
        .into()
}

#[post("/calendars/<calendar_id>/events", data = "<event>")]
pub fn insert_event(mut db: PgsqlConn, calendar_id: UuidParam, event: Json<EventPlain>) -> RouteResult<EventPlain>
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
pub fn update_event(mut db: PgsqlConn, calendar_id: UuidParam, event_id: UuidParam, event_data: Json<EventPlain>) -> RouteResult<()>
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

    // Remove the last comma ',' from the query. Panic if
    // the character removed was not a comma.
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

#[get("/calendars/<calendar_id>/events/<event_id>/instances?<since>&<until>")]
pub fn get_instances(
    mut db: PgsqlConn,
    calendar_id: UuidParam,
    event_id: UuidParam,
    since: Option<NaiveDateParam>,
    until: Option<NaiveDateParam>,
    common_params: CommonQueryParams,
) -> RouteResult<Vec<EventPlain>>
{
    if let Some(event) = get_event_by_id(&mut db, calendar_id, event_id)?
    {
        match event
        {
            Event::Recurring(event) => RouteResult::Ok(

                event
                    .generate_instances(
                        since.map(|x| x.into_inner()),
                        until.map(|x| x.into_inner()),
                        common_params.offset() as usize,
                        common_params.page_size() as usize
                    )?
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

#[get("/calendars/<calendar_id>/events?<since>&<until>")]
pub fn list_events(
    mut db: PgsqlConn,
    calendar_id: UuidParam,
    since: Option<NaiveDateOrTime>,
    until: Option<NaiveDateOrTime>,
    common_params: CommonQueryParams,
) -> RouteResult<Vec<EventPlain>>
{
    // since and until can only be date or date-times
    if (since.is_some() && since.as_ref().unwrap().as_naive_time().is_some())
        || (until.is_some() && until.as_ref().unwrap().as_naive_time().is_some())
    {
        return RouteResult::BadRequest(None);
    }

    /// A query parameter can only have one type and we want to be able to
    /// use `since` and `until` as either a date or a date-time, so we have
    /// a pair of parameters for each variable, one for each type. If, for example,
    /// `since` is a date, `$2` will be `NULL`.
    let query = "
        SELECT * FROM events
        WHERE
            calendar_id = $1
            AND ($2::TIMESTAMP IS NULL OR start_date + start_time >= $2::TIMESTAMP)
            AND ($3::DATE IS NULL OR start_date >= $3::DATE)
            AND ($4::TIMESTAMP IS NULL OR end_date + end_time <= $4::TIMESTAMP)
            AND ($5::DATE IS NULL OR end_date <= $5::DATE)
        OFFSET $6
        LIMIT $7;
    ";

    let rows = db.query(query, &[
        &calendar_id,

        &since.as_ref().and_then(|x| x.as_naive_date_time() .map(|dt| dt.clone())),
        &since.as_ref().and_then(|x| x.as_naive_date()      .map(|d|   d.clone())),

        &until.as_ref().and_then(|x| x.as_naive_date_time() .map(|dt| dt.clone())),
        &until.as_ref().and_then(|x| x.as_naive_date()      .map(|d|   d.clone())),

        &common_params.offset(),
        &common_params.page_size(),
    ]);

    RouteResult::Ok(
        rows?
            .into_iter()
            .map::<Result<EventPlain, _>, _>(|r| Event::from_row(&r).map(|e| e.into_plain()))
            .collect::<Result<Vec<EventPlain>, _>>()?
    )
}

#[get("/calendars/<calendar_id>/events/changes?<since>")]
pub fn check_for_changes(
    mut db: PgsqlConn,
    common_params: CommonQueryParams,
    calendar_id: UuidParam,
    since: NaiveDateOrTime,
) -> RouteResult<Vec<EventPlain>>
{
    if since.as_naive_time().is_some()
    {
        return RouteResult::BadRequest(None);
    }

    let query = "
        SELECT * FROM events
        WHERE
            calendar_id = $1
            AND ($2::TIMESTAMP IS NULL OR last_modified >= $2::TIMESTAMP)
            AND ($3::DATE IS NULL OR last_modified >= $3::DATE)
        OFFSET $4
        LIMIT $5;
    ";

    let rows = db.query(query, &[
        &calendar_id,

        &since.as_naive_date_time(),
        &since.as_naive_date(),


        &common_params.offset(),
        &common_params.page_size(),
    ]);

    RouteResult::Ok (
        rows?
            .into_iter()
            .map::<Result<EventPlain, _>, _>(|r| Event::from_row(&r).map(|e| e.into_plain()))
            .collect::<Result<Vec<EventPlain>, _>>()?
    )
}