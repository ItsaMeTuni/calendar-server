#![feature(proc_macro_hygiene, decl_macro)]
#![feature(try_trait)]
#![feature(backtrace)]

mod connection_pool;
mod database_error;
mod database_helpers;
mod routes;
mod calendar;
mod event;
mod recurrence;

extern crate dotenv;
#[macro_use] extern crate thiserror;
#[macro_use] extern crate rocket;
#[macro_use] extern crate serde;

use crate::connection_pool::PgsqlPool;
use rocket::Request;
use std::env;

fn main()
{
    dotenv::dotenv().ok();

    rocket::ignite()
        .manage(get_pgsql_pool())
        .mount("/api", routes::get_routes())
        .register(catchers![not_found])
        .launch();
}

fn get_pgsql_pool() -> PgsqlPool
{
    let pg_addr: Vec<String> = get_env_default("DB_ADDR", "db:5432")
        .split(':')
        .map(|s| s.to_owned())
        .collect();

    let pg_host = &pg_addr.get(0).expect("Invalid DB_ADDR value.");
    let pg_port = &pg_addr.get(1).expect("Invalid DB_ADDR value.");

    let pg_password = get_env("POSTGRES_PASSWORD");
    let pg_user = get_env("POSTGRES_USER");

    PgsqlPool::new(&format!("host={} port={} dbname={} user={} password={}", pg_host, pg_port, pg_user, pg_user, pg_password))
}

fn get_env(name: &str) -> String
{
    env::vars()
        .find(|(key, _)| key == name)
        .expect(&format!("Missing {} environment variable.", name))
        .1
}

fn get_env_default(name: &str, default: &str) -> String
{
    env::vars()
        .find(|(key, _)| key == name)
        .map(|(_, value)| value)
        .unwrap_or(default.to_owned())
}

#[catch(404)]
fn not_found(_req: &Request) -> () {}