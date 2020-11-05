use rocket::request::{FromRequest, Outcome, Request, State};
use crate::env_helpers::get_env_default;

/// Stores the server's configuration variables.
///
/// Note: if you want to get the page size from inside a route
/// use `CommonQueryParams` instead, which supports the `limit` parameter
/// and falls back to the configured page size.
pub struct Configs
{
    /// Maximum amount of resources a single request can return.
    /// E.g. a List Events request cannot return more events than
    /// the value stored here.
    ///
    /// Generally used as a LIMIT clause in SQL queries.
    page_size: u32,
}

impl Configs
{
    pub fn get_page_size(&self) -> u32
    {
        self.page_size
    }

    pub fn get_configs() -> Configs
    {
        Configs {
            page_size: get_env_default("PAGE_SIZE", "1000").parse().expect("PAGE_SIZE is not a positive integer."),
        }
    }
}