#![feature(proc_macro_hygiene, decl_macro)]
#![feature(try_trait)]
#![feature(backtrace)]
#![allow(dead_code)]

mod connection_pool;
mod database_error;
mod database_helpers;
mod routes;
mod calendar;
mod event;
mod recurrence;
mod configs;
mod env_helpers;
mod iter_helpers;

extern crate dotenv;
#[macro_use] extern crate thiserror;
#[macro_use] extern crate rocket;
#[macro_use] extern crate serde;
#[macro_use] extern crate rocket_okapi;

use crate::connection_pool::PgsqlPool;
use rocket::Request;
use env_helpers::{get_env, get_env_default};
use crate::configs::Configs;
use rocket_okapi::swagger_ui::{make_swagger_ui, SwaggerUIConfig};

fn main()
{
    dotenv::dotenv().ok();

    rocket::ignite()
        .manage(get_pgsql_pool())
        .manage(Configs::get_configs())
        .mount("/api", routes::get_routes())
        .mount(
            "/swagger-ui/",
            make_swagger_ui(&SwaggerUIConfig {
                url: "/api/openapi.json".to_owned(),
                ..Default::default()
            }),
        )
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

    println!("Connecting to DB at {}:{}", pg_host, pg_port);

    PgsqlPool::new(&format!("host={} port={} dbname={} user={} password={}", pg_host, pg_port, pg_user, pg_user, pg_password))
}



#[catch(404)]
fn not_found(_req: &Request) -> () {}