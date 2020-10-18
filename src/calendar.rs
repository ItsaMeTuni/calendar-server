use crate::database_helpers::{FromRow, get_cell_from_row};
use postgres::Row;
use crate::database_error::DatabaseError;

pub const CALENDAR_FIELDS: &str = "id, tenant_id";

#[derive(Debug, Serialize, Deserialize)]
pub struct Calendar
{
    /// If this is -1 it means the calendar does not
    /// exist in the database. Useful when deserializing
    /// Calendar for create requests.
    #[serde(default = "Calendar::default_id")]
    id: i32,
}


impl Calendar
{
    pub fn new(id: i32) -> Calendar
    {
        Calendar {
            id
        }
    }

    pub fn get_id(&self) -> i32 { self.id }

    fn default_id() -> i32 { -1 }
}

impl FromRow for Calendar
{
    type SelfType = Calendar;

    fn from_row(row: &Row) -> Result<Self::SelfType, DatabaseError>
    {
        Ok (
            Calendar {
                id: get_cell_from_row(row, "id")?
            }
        )
    }
}