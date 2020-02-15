//! Documentation for graph module
//!
//! Most functions are called from the `actix-web` framework
use crate::haak::database;

use actix::Addr;
use actix_redis::RedisActor;
use actix_session::Session;
use actix_web::{web::Data, HttpResponse, Result};

use askama::Template;

#[derive(Template)]
#[template(path = "index.html")]
pub struct GraphSettings<'a> {
    temperature: &'a str,
    pressure: &'a str,
    theme: &'a str,
    timeframe: &'a str,
}

/// Index of the graph, if not logged in redirect user to /login
///
/// # Remarks
///
/// Should only be called from actix_web
pub async fn graph_index(session: Session, redis: Data<Addr<RedisActor>>) -> Result<HttpResponse> {
    let user = session.get::<String>("email").unwrap();

    // If not logged in -> redirect to /login
    if user.is_none() {
        return Ok(HttpResponse::SeeOther()
            .header(actix_web::http::header::LOCATION, "/login")
            .body(""));
    }

    let sett =
        database::settings_get(&session.get::<String>("email").unwrap().unwrap(), &redis).await;

    let view = GraphSettings {
        temperature: &sett[0],
        pressure: &sett[1],
        theme: &sett[2],
        timeframe: &sett[3],
    }
    .render()
    .unwrap();

    Ok(HttpResponse::Ok().content_type("text/html").body(view))
}

// TODO: Implement more routes based on what GUI wants.
