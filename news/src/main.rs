#[macro_use]
extern crate actix_web;

#[macro_use]
extern crate serde_json;

use crate::utils::*;
use core::models::dao::EventDao;
use core::models::dto::EventDto;

use actix_web::http::StatusCode;
use actix_web::middleware::errhandlers::{ErrorHandlerResponse, ErrorHandlers};
use actix_web::{web, App, HttpResponse, HttpServer, Result};
use diesel::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;

use std::io;

use askama::Template;

mod utils;

#[derive(Template)]
#[template(path = "index.html")]
struct UserTemplate {
    offset: usize,
    next_offset: usize,
    events: Vec<EventDto>,
}

fn get_events(pool: Arc<Arc<Pool>>, offset: i64) -> Vec<EventDto> {
    use core::schema::events::dsl::{created_at, events};

    let conn: &PgConnection = &(pool.get()).unwrap();

    let dao = events
        .order_by(created_at.desc())
        .offset(offset)
        .limit(20)
        .load::<EventDao>(conn)
        .unwrap();

    dao.into_iter().map(|w| w.into()).collect()
}

async fn index(
    pool: web::Data<Arc<Pool>>,
    query: web::Query<HashMap<String, String>>,
) -> Result<HttpResponse> {
    let n = 20;
    let s = if let Some(name) = query.get("offset") {
        let offset = name
            .parse()
            .map_err(|_| HttpResponse::BadRequest().json("wrong offset"))?;

        UserTemplate {
            offset,
            next_offset: offset + n,
            events: Vec::new(),
        }
        .render()
        .unwrap()
    } else {
        let events = get_events(pool.into_inner(), 0);
        UserTemplate {
            offset: 0,
            next_offset: n,
            events,
        }
        .render()
        .unwrap()
    };
    Ok(HttpResponse::Ok().content_type("text/html").body(s))
}

#[actix_rt::main]
async fn main() -> io::Result<()> {
    dotenv::dotenv().ok();
    std::env::set_var("RUST_LOG", "info,actix_web=info,actix_server=info");
    env_logger::init();

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL expected");
    let pool_pg = connect_pg(database_url);

    HttpServer::new(move || {
        App::new()
            .data(Arc::new(pool_pg.clone()))
            .service(web::resource("/").route(web::get().to(index)))
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}
