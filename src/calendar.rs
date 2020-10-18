pub const CALENDAR_FIELDS: &str = "id, tenant_id";

#[derive(Debug)]
pub struct Calendar
{
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
}