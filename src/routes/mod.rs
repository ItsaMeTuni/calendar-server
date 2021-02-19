//! TODO: protect all routes with an ApiKey request guard automatically

use rocket::Route;
use rocket_okapi::routes_with_openapi;

mod routes_calendar;
mod routes_event;
mod common_query_params;

/// All project routes go in here, main.rs
/// uses this method to get all routes.
pub fn get_routes() -> Vec<Route>
{
    routes_with_openapi![
        routes_calendar::get_calendar,
        routes_calendar::insert_calendar,
        routes_calendar::list_calendars,

        routes_event::get_event,
        routes_event::insert_event,
        routes_event::get_instances,
        routes_event::update_event,
        routes_event::list_events,
        routes_event::check_for_changes,
    ]
}