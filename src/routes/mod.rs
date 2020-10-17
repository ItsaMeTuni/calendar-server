use std::error::Error;
use rocket::response::Responder;
use rocket::{Request, Response, Route};
use rocket_contrib::json::Json;
use rocket::http::{Status, ContentType};
use serde_json::Value;
use serde::Serialize;
use std::ops::{Deref, Try};
use std::io::Cursor;

mod routes_calendar;

pub fn get_routes() -> Vec<Route>
{
    routes![
    ]
}


pub enum RouteResult<T>
    where T: Serialize
{
    Ok(T),
    Created,
    NotFound,
    BadRequest(Option<Box<dyn Serializable>>),
    Forbidden,
    InternalError(Box<dyn Error>),
}

impl<'r, T: Serialize> Responder<'r> for RouteResult<T>
{
    fn respond_to(self, request: &Request) -> rocket::response::Result<'r>
    {
        let mut response = Response::new();

        let status = match self
        {
            RouteResult::Ok(_) => Status::Ok,
            RouteResult::Created => Status::Created,
            RouteResult::NotFound => Status::NotFound,
            RouteResult::BadRequest(_) => Status::BadRequest,
            RouteResult::Forbidden => Status::Forbidden,
            RouteResult::InternalError(_) => Status::InternalServerError,
        };

        response.set_status(status);

        let body = match &self
        {
            RouteResult::Ok(payload) => Some(serde_json::to_string(payload)),
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
        else
        {
            return Err(Status::InternalServerError);
        }

        Ok(response)
    }
}

pub trait Serializable
{
    fn serialize_json(&self) -> serde_json::Result<String>;
}

impl<T> Serializable for T
    where T: Serialize
{
    fn serialize_json(&self) -> serde_json::Result<String>
    {
        serde_json::to_string(self)
    }
}

impl<T, E: 'static> Into<RouteResult<T>> for Result<Option<T>, E>
    where
        T: Serialize,
        E: Error,
{
    fn into(self) -> RouteResult<T>
    {
        match self
        {
            Ok(opt) => match opt
            {
                Some(payload) => RouteResult::Ok(payload),
                None => RouteResult::NotFound,
            },
            Err(e) => RouteResult::InternalError(Box::new(e)),
        }
    }
}