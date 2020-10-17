#![feature(proc_macro_hygiene, decl_macro)]
#![feature(try_trait)]
#![feature(backtrace)]

mod connection_pool;
mod database_error;
mod database_helpers;
mod calendar;
mod routes;

#[macro_use] extern crate thiserror;
#[macro_use] extern crate rocket;
#[macro_use] extern crate serde;

use crate::connection_pool::PgsqlPool;
use rocket::Request;

fn main()
{
    let pool = PgsqlPool::new("host=localhost dbname=calendar_app user=calendar_app_backend password=changeme");

    rocket::ignite()
        .manage(pool)
        .mount("/api", routes::get_routes())
        .register(catchers![not_found])
        .launch();
}

#[catch(404)]
fn not_found(_req: &Request) -> () {}