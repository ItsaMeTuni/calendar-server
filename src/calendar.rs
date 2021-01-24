use crate::database_helpers::{FromRow, get_cell_from_row};
use postgres::Row;
use crate::database_error::DatabaseError;
use uuid::Uuid;

pub const CALENDAR_FIELDS: &str = "id, tenant_id";

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct Calendar
{
    /// If this is a nil UUID (it's a thing, look it up)
    /// it means the calendar does not exist in the database.
    /// This is useful when deserializing Calendar
    /// for create requests.
    #[serde(default = "Uuid::nil")]
    id: Uuid,
}


impl Calendar
{
    pub fn new(id: Uuid) -> Calendar
    {
        Calendar {
            id
        }
    }

    pub fn get_id(&self) -> Uuid { self.id }
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