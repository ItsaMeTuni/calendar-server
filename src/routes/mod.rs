use std::error::Error;
use rocket::response::Responder;
use rocket::{Request, Response, Route};

use rocket::http::{Status, ContentType};

use serde::Serialize;
use std::ops::{Deref, Try};
use std::io::Cursor;
use std::fmt::Debug;
use rocket::http::hyper::header::Location;
use std::any::Any;
use rocket_okapi::response::OpenApiResponder;
use rocket_okapi::gen::OpenApiGenerator;
use okapi::openapi3::Responses;
use rocket_okapi::util::{add_schema_response, produce_any_responses};
use schemars::JsonSchema;
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


/// Every route in the project should return this,
/// it implements Responder.
///
/// If you don't want to return anything in the response's
/// body just use RouteResult<()>.
#[derive(Debug)]
pub enum RouteResult<T>
    where T: Serialize
{
    /// 200 Ok
    /// Sends {0} in the response body.
    Ok(T),

    /// 201 Created
    /// Sends {0} in the response body, sets
    /// the response Location header as {1}.
    Created(T, String),

    /// 404 Not Found
    NotFound,

    /// 400 Bad requests, serializes payload (if present)
    /// and sends as response body. This payload is
    /// used to supply details to the client.
    BadRequest(Option<Box<dyn Serializable>>),

    /// 401 Unauthorized
    Forbidden,

    /// 500 Internal Server Error
    /// Logs the payload, does NOT send it to the
    /// client.
    InternalError(Box<dyn Error>),
}



/// Transforms a RouteResult into a response with the appropriate
/// status code and body.
impl<'r, T: Serialize + Debug + 'static> Responder<'r> for RouteResult<T>
{
    fn respond_to(self, _request: &Request) -> rocket::response::Result<'r>
    {
        let mut response = Response::new();

        let status = match self
        {
            RouteResult::Ok(_) => Status::Ok,
            RouteResult::Created(_, _) => Status::Created,
            RouteResult::NotFound => Status::NotFound,
            RouteResult::BadRequest(_) => Status::BadRequest,
            RouteResult::Forbidden => Status::Forbidden,
            RouteResult::InternalError(_) => Status::InternalServerError,
        };

        response.set_status(status);

        let body = match &self
        {
            RouteResult::Ok(payload) =>
            {
                // Don't serialize payload if it's type is `()`
                if (payload as &dyn Any).downcast_ref::<()>().is_some()
                {
                    None
                }
                else
                {
                    Some(serde_json::to_string(payload))
                }
            },
            RouteResult::Created(payload, _) => Some(serde_json::to_string(payload)),
            RouteResult::BadRequest(payload) => payload.as_ref().map(|x| x.deref().serialize_json()),
            _ => None,
        };

        match self
        {
            RouteResult::InternalError(e) =>
            {
                eprintln!("{}", e);
                if let Some(backtrace) = e.backtrace()
                {
                    eprintln!("{}", backtrace);
                }
                else
                {
                    eprintln!("No backtrace available");
                }
            },
            RouteResult::Created(_, location) => { response.set_header(Location(location)); },
            _ => {},
        }

        if let Some(body) = body
        {
            if body.is_ok()
            {
                response.set_header(ContentType::JSON);
                response.set_sized_body(Cursor::new(body.unwrap()));
            }
            else
            {
                return Err(Status::InternalServerError);
            }
        }

        Ok(response)
    }
}

impl<T: Serialize + Debug + JsonSchema + 'static> OpenApiResponder<'_> for RouteResult<T>
{
    fn responses(gen: &mut OpenApiGenerator) -> rocket_okapi::Result<Responses> {
        let mut success_responses = Responses::default();
        let success_schema = gen.json_schema::<T>();
        add_schema_response(&mut success_responses, 200, "application/json", success_schema.clone())?;
        add_schema_response(&mut success_responses, 201, "application/json", success_schema)?;

        let mut err_responses = Responses::default();
        let empty_schema = gen.json_schema::<()>();
        add_schema_response(&mut err_responses, 404, "", empty_schema.clone())?;
        add_schema_response(&mut err_responses, 401, "", empty_schema.clone())?;
        add_schema_response(&mut err_responses, 400, "", empty_schema.clone())?;
        add_schema_response(&mut err_responses, 500, "", empty_schema)?;

        produce_any_responses(success_responses, err_responses)
    }
}


/// Error to be used in RouteResult::into_result, nowhere else.
#[derive(Error, Debug)]
#[error("Not found")]
struct NotFoundError;

/// Error to be used in RouteResult::into_result, nowhere else.
#[derive(Error, Debug)]
#[error("Forbidden")]
struct ForbiddenError;

/// Error to be used in RouteResult::into_result, nowhere else.
#[derive(Error, Debug)]
#[error("Bad Request")]
struct BadRequestError;

/// Allow the use of the ? operator on RouteResult.
///
///
/// - Ok(payload)) gets turned into Ok(Some(payload))
/// - Created -> Ok(None)
/// - All others are turned into errors.
impl<T> Try for RouteResult<T>
    where T: Serialize
{
    type Ok = Option<T>;
    type Error = Box<dyn Error>;

    fn into_result(self) -> Result<<RouteResult<T> as Try>::Ok, Self::Error>
    {
        match self
        {
            RouteResult::Ok(x) => Ok(Some(x)),
            RouteResult::Created(payload, _) => Ok(Some(payload)),
            RouteResult::NotFound => Err(Box::new(NotFoundError)),
            RouteResult::BadRequest(_x) => Err(Box::new(BadRequestError)),
            RouteResult::Forbidden => Err(Box::new(ForbiddenError)),
            RouteResult::InternalError(x) => Err(x),
        }
    }

    fn from_error(v: Self::Error) -> Self
    {
        RouteResult::InternalError(v)
    }

    fn from_ok(v: <RouteResult<T> as Try>::Ok) -> Self
    {
        match v
        {
            Some(x) => RouteResult::Ok(x),
            None => RouteResult::NotFound,
        }
    }
}

impl<T, E: 'static> From<Result<Option<T>, E>> for RouteResult<T>
    where
        T: Serialize,
        E: Error
{
    fn from(result: Result<Option<T>, E>) -> Self
    {
        match result
        {
            Ok(x) => match x
            {
                Some(payload) => RouteResult::Ok(payload),
                None => RouteResult::NotFound,
            },
            Err(e) => RouteResult::InternalError(Box::new(e))
        }
    }
}

/// Convenience trait that has a method for easy
/// json serialization.
///
/// This is also needed because Box<dyn Serialize + Debug> is a compilation
/// error, so we join those two in this trait. The reason for Serialize not
/// a supertrait of this is because it has methods with generic parameters,
/// so it can't be made into an object.
pub trait Serializable: Debug
{
    fn serialize_json(&self) -> serde_json::Result<String>;
}

impl<T> Serializable for T
    where T: Serialize + Debug
{
    fn serialize_json(&self) -> serde_json::Result<String>
    {
        serde_json::to_string(self)
    }
}