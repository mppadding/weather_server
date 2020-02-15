//! Documentation for settings module
//!
//! Most functions are called from the `actix-web` framework
use crate::haak::database;

use actix::Addr;
use actix_redis::RedisActor;
use actix_session::Session;
use actix_web::web::{Data, Form};
use actix_web::{HttpResponse, Result};

use serde::Deserialize;

use askama::Template;

#[derive(Template)]
#[template(path = "settings.html")]
pub struct Settings<'a> {
    temperature: &'a str,
    pressure: &'a str,
    theme: &'a str,
    timeframe: &'a str,
    admin: bool,
}

/// Shows settings index. If the user is an admin it also shows the registration form. Redirects to
/// /login if not logged in.
///
/// # Remarks
///
/// Should only be called from actix_web
pub async fn settings_index(
    session: Session,
    redis: Data<Addr<RedisActor>>,
) -> Result<HttpResponse> {
    let user = session.get::<String>("email").unwrap();

    // If not logged in -> redirect to /login
    if user.is_none() {
        return Ok(HttpResponse::SeeOther()
            .header(actix_web::http::header::LOCATION, "/login")
            .body(""));
    }

    let sett =
        database::settings_get(&session.get::<String>("email").unwrap().unwrap(), &redis).await;

    let view = Settings {
        temperature: &sett[0],
        pressure: &sett[1],
        theme: &sett[2],
        timeframe: &sett[3],
        admin: database::user_is_admin(&user.unwrap(), &redis).await,
    }
    .render()
    .unwrap();

    Ok(HttpResponse::Ok().content_type("text/html").body(view))
}

/// Form data returned from settings-save
#[derive(Deserialize, Debug)]
pub struct SettingsData {
    pub temperature: String,
    pub pressure: String,
    pub theme: String,
    pub timeframe: String,
}

/// Settings validator.
/// Returns true if settings are valid.
///
/// # Arguments
///
/// * `data` - SettingsData containing all settings
fn validate_settings(data: &SettingsData) -> bool {
    return (data.temperature == "Celsius"
        || data.temperature == "Kelvin"
        || data.temperature == "Fahrenheit")
        && (data.pressure == "Atmosphere"
            || data.pressure == "Millibar"
            || data.pressure == "Bar"
            || data.pressure == "PSI"
            || data.pressure == "Mercury")
        && (data.theme == "Light" || data.theme == "Dark")
        && (data.timeframe == "Week"
            || data.timeframe == "Month"
            || data.timeframe == "QuarterYear");
}

/// Handles POST requests to /settings. Saves the settings in the database.
/// Redirects to /login if not logged in.
///
/// # Arguments
///
/// * `form` - JSON data of the settings form
/// * `session` - Session containing all CookieSession data
/// * `redis` - RedisActor to access redis database
///
/// # Remarks
///
/// Should only be called from actix_web
pub async fn settings_save(
    form: Form<SettingsData>,
    session: Session,
    redis: Data<Addr<RedisActor>>,
) -> HttpResponse {
    // If not logged in -> redirect to /login
    if session.get::<String>("email").unwrap().is_none() {
        return HttpResponse::SeeOther()
            .header(actix_web::http::header::LOCATION, "/login")
            .finish();
    }

    let data = SettingsData {
        temperature: form.temperature.clone(),
        pressure: form.pressure.clone(),
        theme: form.theme.clone(),
        timeframe: form.timeframe.clone(),
    };

    if validate_settings(&data) {
        database::settings_set(
            &session.get::<String>("email").unwrap().unwrap(),
            &data,
            &redis,
        )
        .await;
    }

    return HttpResponse::SeeOther()
        .header(actix_web::http::header::LOCATION, "/settings")
        .finish();
}
