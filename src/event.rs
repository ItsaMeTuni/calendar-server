//! A few notes:
//!
//! Everything is stored in UTC: `NaiveDate`s and `NaiveTime`s are all in UTC,
//! and the DATEs and TIMEs in the database are in UTC (and have no timezone).

use crate::connection_pool::PgsqlConn;
use crate::database_error::{DatabaseError, DatabaseErrorKind};
use postgres::Row;
use crate::database_helpers::{get_cell_from_row, get_cell_from_row_with_default};
use chrono::{Date, DateTime, TimeZone, NaiveDate, NaiveDateTime, Utc, NaiveTime, Duration};
use crate::recurrence::RecurrenceRule;


use serde::{Serialize, Deserialize};




pub const EVENT_FIELDS: &str = "id, parent_event_id, start_date, start_time, end_date, end_time, rrule, exdates, rdates";

#[derive(Copy, Clone, Debug)]
pub struct EventDateSpan
{
    start: NaiveDate,
    end: NaiveDate,
}







#[derive(Copy, Clone, Debug)]
pub struct EventDateTimeSpan
{
    start: NaiveDateTime,
    end: NaiveDateTime,
}

impl EventDateTimeSpan
{
    pub fn as_date_span(&self) -> EventDateSpan
    {
        EventDateSpan {
            start: self.start.date(),
            end: self.end.date(),
        }
    }
}




#[derive(Copy, Clone, Debug)]
pub enum EventSpan
{
    Date(EventDateSpan),
    DateTime(EventDateTimeSpan),
}

impl EventSpan
{
    pub fn get_date_span(&self) -> EventDateSpan
    {
        match self
        {
            EventSpan::Date(date_span) => *date_span,
            EventSpan::DateTime(datetime_span) => datetime_span.as_date_span(),
        }
    }

    pub fn get_date_time_span(&self) -> Option<EventDateTimeSpan>
    {
        match self
        {
            EventSpan::Date(_) => None,
            EventSpan::DateTime(datetime_span) => Some(*datetime_span),
        }
    }

    pub fn get_start_date(&self) -> NaiveDate
    {
        self.get_date_span().start
    }

    pub fn get_end_date(&self) -> NaiveDate
    {
        self.get_date_span().end
    }

    pub fn get_start_time(&self) -> Option<NaiveTime>
    {
        self.get_date_time_span().map(|dt| dt.start.time())
    }

    pub fn get_end_time(&self) -> Option<NaiveTime>
    {
        self.get_date_time_span().map(|dt| dt.end.time())
    }

    pub fn get_duration(&self) -> Duration
    {
        match self
        {
            EventSpan::Date(date_span) => date_span.end - date_span.start,
            EventSpan::DateTime(datetime_span) => datetime_span.end - datetime_span.start,
        }
    }

    pub fn from_date_and_duration(start: NaiveDate, duration: Duration) -> EventSpan
    {
        EventSpan::Date(
            EventDateSpan {
                start,
                end: start + duration,
            }
        )
    }

    pub fn from_date_time_and_duration(start: NaiveDateTime, duration: Duration) -> EventSpan
    {
        EventSpan::DateTime(
            EventDateTimeSpan {
                start,
                end: start + duration,
            }
        )
    }

    /// Constructs an EventSpan from a query result Row
    /// that has the columns start_date, end_date, start_time, and end_time
    fn from_row(row: &Row) -> Result<Self, DatabaseError>
    {
        let start_date: NaiveDate = row.try_get("start_date")?;
        let end_date: NaiveDate = row.try_get("end_date")?;

        // start_time and end_time might be NULL in the database, if they are
        // we create a Date interval, if they're not we create a DateTime interval.
        if let Some(start_time) = row.try_get::<_, Option<NaiveTime>>("start_time")?
        {
            // Btw, there is (or should be) a DB constraint that prevents only
            // one of start_time and end_time from being NULL, it's either both or none.
            let end_time = row.try_get::<_, Option<NaiveTime>>("end_time")?
                .ok_or(DatabaseErrorKind::UnexpectedNull("end_time".to_owned()))?;

            Ok(
                EventSpan::DateTime(EventDateTimeSpan {
                    start: NaiveDateTime::new(start_date, start_time),
                    end: NaiveDateTime::new(end_date, end_time),
                })
            )
        }
        else
        {
            Ok(
                EventSpan::Date(EventDateSpan {
                    start: start_date,
                    end: end_date,
                })
            )
        }
    }
}






#[derive(Clone, Debug)]
pub struct EventRecurrence
{
    rule: RecurrenceRule,
    exdates: Vec<NaiveDate>,
    rdates: Vec<NaiveDate>,
}








#[derive(Debug)]
pub enum Event
{
    Recurring(EventRecurring),
    Single(EventSingle),
}

impl Event
{
    pub fn from_row(row: &Row) -> Result<Self, DatabaseError>
    {
        if get_cell_from_row::<Option<String>>(row, "rrule")?.is_some()
        {
            Ok(Event::Recurring(EventRecurring::from_row(row)?))
        }
        else
        {
            Ok(Event::Single(EventSingle::from_row(row)?))
        }
    }
}

impl ToPlain<EventPlain> for Event
{
    fn into_plain(self) -> EventPlain
    {
        match self
        {
            Event::Recurring(e) => e.into_plain(),
            Event::Single(e) => e.into_plain(),
        }
    }
}

impl ToPlain<Vec<EventPlain>> for Vec<Event>
{
    fn into_plain(self) -> Vec<EventPlain>
    {
        self
            .into_iter()
            .map(|x| x.into_plain())
            .collect()
    }
}






#[derive(Clone, Debug)]
pub struct EventRecurring
{
    /// Id of this event in the database.
    id: i32,
    span: EventSpan,
    recurrence: EventRecurrence,
}

/// If you want to get an event you have to get it from
/// its calendarold.
impl EventRecurring
{
    pub fn get_id(&self) -> i32 { self.id }

    pub fn get_span(&self) -> EventSpan { self.span }

    pub fn get_recurrence(&self) -> EventRecurrence { self.recurrence.clone() }

    /// Get all events in the database that have this event's id
    /// as their parent_id. The result is ordered by start date.
    ///
    /// Does **NOT** generate events based on the recurrence! Use `generate_instances`
    /// instead!
    pub fn get_children(&self, db: &mut PgsqlConn) -> Result<Vec<EventSingle>, DatabaseError>
    {
        let query = format!("SELECT {} FROM events WHERE parent_id = $1 ORDER BY start_date;", EVENT_FIELDS);

        let rows = db.query(query.as_str(), &[&self.id])?;

        rows.iter()
            .map(|x| EventSingle::from_row(x))
            .collect()
    }

    /// Generates event instances between dates `from_date` and `to_date` (both inclusive)
    /// based on this event's rrule, exdates, rdates.
    ///
    /// Does **NOT** get child events! Use `get_children` for that!
    pub fn generate_instances(&self, from_date: NaiveDate, to_date: NaiveDate) -> Result<Vec<EventInstance>, DatabaseError>
    {
        let mut instances = self.recurrence.rule.calculate_instances(from_date, to_date, self.span.get_date_span().start);

        instances.append(&mut self.recurrence.rdates.clone());

        let duration = self.span.get_duration();

        Ok(
            instances
                .iter()
                .filter(|x| !self.recurrence.exdates.contains(*x))
                .map(|date| {
                    EventInstance {
                        parent_id: self.id,
                        span: match self.span
                        {
                            EventSpan::Date(_date_span) => EventSpan::from_date_and_duration(*date, duration),
                            EventSpan::DateTime(datetime_span) => EventSpan::from_date_time_and_duration(date.and_time(datetime_span.start.time()), duration),
                        },
                    }
                })
                .collect()
        )
    }

    pub fn from_row(row: &Row) -> Result<Self, DatabaseError>
    {
        let span = EventSpan::from_row(row)?;

        let rrule_field: Option<&str> = get_cell_from_row(row, "rrule")?;

        if rrule_field.is_none()
        {
            panic!("Tried making an EventRecurrent from non-recurrent event.");
        }

        let recurrence_rule = RecurrenceRule::new(rrule_field.unwrap(), span.get_start_date())
            .map_err(|e| DatabaseErrorKind::Other(Box::new(e)))?;

        let recurrence = EventRecurrence {
            exdates: get_cell_from_row_with_default(row, "exdates",  vec![])?,
            rdates: get_cell_from_row_with_default(row, "rdates",  vec![])?,
            rule: recurrence_rule,
        };

        Ok(
            EventRecurring {
                id: get_cell_from_row(row, "id")?,
                span,
                recurrence,
            }
        )
    }
}

impl ToPlain<EventPlain> for EventRecurring
{
    fn into_plain(self) -> EventPlain
    {
        EventPlain {
            id: Some(self.id),
            parent_id: None,

            start_date: self.span.get_start_date(),
            end_date: self.span.get_end_date(),
            start_time: self.span.get_start_time(),
            end_time: self.span.get_end_time(),

            recurrence: Some(
                RecurrencePlain {
                    rrule: self.recurrence.rule.to_string(),
                    exdates: self.recurrence.exdates,
                    rdates: self.recurrence.rdates,
                }
            ),
        }
    }
}









#[derive(Clone, Debug)]
pub struct EventSingle
{
    id: i32,

    /// If this is Some, it means this event is a single
    /// event that was originated from modifying the date/time
    /// of an event instance. That event instance does not exist
    /// anymore (i.e. won't be generated by `EventRecurring::generate_instances`)
    /// and this one "took its place".
    ///
    /// If this is None it just means this is a non-recurring event,
    /// without any relationship with any other event in the calendarold.
    ///
    /// For example:
    ///
    /// Imagine there's a recurrent event that starts at 2020-09-01 (Tue),
    /// happens weekly (every Tuesday), and has an ID of `abc`.
    /// Now imagine the user decided to move the instance of 2020-09-08
    /// one day ahead, making it happen on 2020-09-09.
    /// What happened "behind the scenes" is:
    /// 1. The date 2020-09-08 was added to the recurrent event's EXDATES property.
    /// 2. A (non-recurring) event was created at 2020-09-09, with the ID `cde`.
    /// 3. The parent_id of the `cde` event was set to `abc`.
    parent_id: Option<i32>,
    span: EventSpan,
}

impl EventSingle
{
    pub fn get_span(&self) -> EventSpan { self.span }

    pub fn get_id(&self) -> i32 { self.id }

    pub fn get_parent_id(&self) -> Option<i32> { self.parent_id }

    pub fn to_plain(&self) -> EventPlain
    {
        EventPlain {
            id: Some(self.id),
            parent_id: self.parent_id,

            start_date: self.span.get_start_date(),
            end_date: self.span.get_end_date(),
            start_time: self.span.get_start_time(),
            end_time: self.span.get_end_time(),

            recurrence: None,
        }
    }

    pub fn from_row(row: &Row) -> Result<Self, DatabaseError>
    {
        Ok(
            EventSingle {
                id: get_cell_from_row(row, "id")?,
                parent_id: get_cell_from_row(row, "parent_event_id")?,
                span: EventSpan::from_row(row)?,
            }
        )
    }
}

impl ToPlain<EventPlain> for EventSingle
{
    fn into_plain(self) -> EventPlain
    {
        EventPlain {
            id: Some(self.id),
            parent_id: self.parent_id,

            start_date: self.span.get_start_date(),
            end_date: self.span.get_end_date(),
            start_time: self.span.get_start_time(),
            end_time: self.span.get_end_time(),

            recurrence: None,
        }
    }
}







#[derive(Clone, Debug)]
pub struct EventInstance
{
    parent_id: i32,
    span: EventSpan,
}

impl EventInstance
{
    pub fn get_span(&self) -> EventSpan { self.span }

    pub fn get_parent_id(&self) -> i32 { self.parent_id }

    pub fn from_row(row: &Row) -> Result<Self, DatabaseError>
    {
        Ok(
            EventInstance {
                parent_id: get_cell_from_row(row, "parent_id")?,
                span: EventSpan::from_row(row)?,
            }
        )
    }
}

impl ToPlain<EventPlain> for EventInstance
{
    fn into_plain(self) -> EventPlain
    {
        EventPlain {
            id: None,
            parent_id: Some(self.parent_id),

            start_date: self.span.get_start_date(),
            end_date: self.span.get_end_date(),
            start_time: self.span.get_start_time(),
            end_time: self.span.get_end_time(),

            recurrence: None,
        }
    }
}


/// This is a serializable representation of an event
/// (single, recurrent or instance), it has two purposes:
/// 1) sending events to the client; 2) receiving events
/// from the client, validating them, and sending them to
/// the database. Nothing else.
///
///
/// Single events don't have an rrule.
/// Recurring events have an rrule value.
/// Instance events don't have an id and have a parent id.
///
///
/// If you want to create an EventPlain, call `to_plain`
/// on an `EventSingle`, `EventInstance` or `EventRecurring`.
///
/// You cannot create an `EventSingle`, `EventInstance` or `EventRecurring`
/// directly from an `EventPlain`. This is by design, there should
/// be no need to modify an incoming event before writing
/// it to the database, only validate it. You _can_ modify
/// it since all fields are public, but it SHOULD NOT be done
/// and you won't have any of the convenience functions the
/// other event structs provide. **Only modify fields directly
/// if you know what you're doing.**
#[derive(Serialize, Deserialize, Debug)]
pub struct EventPlain
{
    pub id: Option<i32>,
    pub parent_id: Option<i32>,

    #[serde(with = "event_plain_serde::date")]
    pub start_date: NaiveDate,

    #[serde(with = "event_plain_serde::time_option")]
    pub start_time: Option<NaiveTime>,

    #[serde(with = "event_plain_serde::date")]
    pub end_date: NaiveDate,

    #[serde(with = "event_plain_serde::time_option")]
    pub end_time: Option<NaiveTime>,

    pub recurrence: Option<RecurrencePlain>,
}


/// Should only be used in conjunction with EventPlain
#[derive(Serialize, Deserialize, Debug)]
pub struct RecurrencePlain
{
    pub rrule: String,

    #[serde(with = "event_plain_serde::date_vec")]
    pub exdates: Vec<NaiveDate>,

    #[serde(with = "event_plain_serde::date_vec")]
    pub rdates: Vec<NaiveDate>
}

pub trait ToPlain<T: Serialize + Deserialize<'static>>
{
    fn into_plain(self) -> T;
}

mod event_plain_serde
{
    const DATE_FORMAT: &'static str = "%Y-%m-%d";
    const TIME_FORMAT: &'static str = "%H:%M:%S";


    pub mod date
    {
        use chrono::{NaiveDate};
        use serde::{self, Deserialize, Serializer, Deserializer};
        
        use super::DATE_FORMAT;

        pub fn serialize<S>(date: &NaiveDate, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
        {
            let string = format!("{}", date.format(DATE_FORMAT));
            serializer.serialize_str(&string)
        }

        pub fn deserialize<'de, D>(deserializer: D) -> Result<NaiveDate, D::Error>
            where
                D: Deserializer<'de>,
        {
            let string = String::deserialize(deserializer)?;

            NaiveDate::parse_from_str(&string, DATE_FORMAT)
                .map_err(serde::de::Error::custom)
        }
    }

    pub mod date_vec
    {
        use chrono::{NaiveDate};
        use serde::{self, Deserialize, Serializer, Deserializer};
        
        use serde::ser::SerializeSeq;
        use super::DATE_FORMAT;

        pub fn serialize<S>(dates: &Vec<NaiveDate>, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
        {
            let mut seq = serializer.serialize_seq(Some(dates.len()))?;

            for date in dates
            {
                seq.serialize_element(&format!("{}", date.format(DATE_FORMAT)));
            }

            seq.end()
        }

        pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<NaiveDate>, D::Error>
            where
                D: Deserializer<'de>,
        {
            let vec = Vec::<String>::deserialize(deserializer)?;

            vec.into_iter()
                .map(|x| NaiveDate::parse_from_str(&x, DATE_FORMAT))
                .collect::<Result<Vec<NaiveDate>, _>>()
                .map_err(serde::de::Error::custom)
        }
    }

    pub mod time_option
    {
        use chrono::{NaiveTime};
        use serde::{self, Deserialize, Serializer, Deserializer};
        
        use super::TIME_FORMAT;

        pub fn serialize<S>(date: &Option<NaiveTime>, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
        {
            match date
            {
                Some(date) => serializer.serialize_str(&format!("{}", date.format(TIME_FORMAT))),
                None => serializer.serialize_none(),
            }
        }

        pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<NaiveTime>, D::Error>
            where
                D: Deserializer<'de>,
        {
            let string = Option::<String>::deserialize(deserializer)?;

            string
                .map(
                    |string| NaiveTime::parse_from_str(&string, TIME_FORMAT)
                        .map_err(serde::de::Error::custom)
                )
                .transpose()
        }
    }
}