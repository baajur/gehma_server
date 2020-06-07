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
    offset: i32,
    next_offset: i32,
    events: Vec<EventDto>,
}

#[derive(Template)]
#[template(path = "item.html")]
struct ItemTemplate {
    event: EventDto,
}

fn get_events(pool: Arc<Arc<Pool>>, offset: i32) -> Vec<EventDto> {
    //use core::schema::events::dsl::{created_at, id, events};
    //use core::schema::votes::dsl::{votes};

    let conn: &PgConnection = &(pool.get()).unwrap();

    let dao: Vec<EventDao> = diesel::sql_query("SELECT * FROM trending OFFSET $1 LIMIT 20")
        .bind::<diesel::sql_types::Integer, _>(offset)
        .load(conn)
        .unwrap();

    dao.into_iter().map(|w| w.into()).collect()
}

fn get_event(pool: Arc<Arc<Pool>>, item: i32) -> EventDto {
    use core::schema::events::dsl::{events, id};

    let conn: &PgConnection = &(pool.get()).unwrap();

    let dto = events
        .filter(id.eq(item))
        .limit(1)
        .load::<EventDao>(conn)
        .unwrap()
        .first()
        .unwrap()
        .clone()
        .into();

    dto
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

        if offset < 0 {
            return Ok(HttpResponse::BadRequest().json("wrong offset"));
        }

        let events = get_events(pool.into_inner(), offset);

        UserTemplate {
            offset,
            next_offset: offset + n,
            events,
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

async fn item(pool: web::Data<Arc<Pool>>, id: web::Path<String>) -> Result<HttpResponse> {
    let item = id
        .into_inner()
        .parse()
        .map_err(|_| HttpResponse::BadRequest().json("Invalid id"))?;

    let event = get_event(pool.into_inner(), item);

    if event.href.is_some() {
        return Ok(HttpResponse::PermanentRedirect()
            .header(actix_web::http::header::LOCATION, event.href.unwrap())
            .finish()
            .into_body());
    }

    let s = ItemTemplate { event: event }.render().unwrap();

    Ok(HttpResponse::Ok().content_type("text/html").body(s))
}

async fn privacy(pool: web::Data<Arc<Pool>>) -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().content_type("text/plain").body("Bei dem Aufruf der Seite werden IP-Adresse, Uhrzeit und Browser mitgespeichert. Diese Daten werden alle 3 Monate wieder geloescht werden."))
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
            .service(web::resource("/privacy").route(web::get().to(privacy)))
            .service(web::resource("item/{id}").route(web::get().to(item)))
            .default_service(web::route().to(|| HttpResponse::NotFound()))
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}
