//! Documentation for database module
//!
//! Most functions are called from the `actix-web` framework
use crate::haak::settings;

use actix::Addr;
use actix_redis::{Command, RedisActor, RespValue};
use actix_web::web::Data;

/// Checks if a user exists in the database.
///
/// # Arguments
///
/// * `email` - Email address to check
/// * `redis` - Connection to database
pub async fn user_exists(email: &String, redis: &Data<Addr<RedisActor>>) -> bool {
    let res = redis
        .send(Command(resp_array!["EXISTS", "user:".to_owned() + &email]))
        .await
        .expect("Database error")
        .unwrap();

    res == RespValue::Integer(1)
}

/// Checks if a user exists in the database.
///
/// # Arguments
///
/// * `email` - Email address to check
/// * `redis` - Connection to database
pub async fn user_is_admin(email: &String, redis: &Data<Addr<RedisActor>>) -> bool {
    let res = redis
        .send(Command(resp_array!["GET", "user:".to_owned() + &email]))
        .await
        .expect("Database error")
        .unwrap();

    res == RespValue::BulkString(vec![97, 100, 109, 105, 110])
}

/// Registers a new user in the system, adds the email and token to the database.
///
/// # Arguments
///
/// * `email` - Email address to register
/// * `token` - Challenge token
/// * `redis` - Connection to database
pub async fn register_email(email: &String, token: &String, redis: &Data<Addr<RedisActor>>) {
    redis
        .send(Command(resp_array![
            "SET",
            "register:".to_owned() + &token,
            email.clone()
        ]))
        .await
        .expect("Database error")
        .unwrap();

    // Set the key to expire in 1 hour
    redis
        .send(Command(resp_array![
            "EXPIRE",
            "register:".to_owned() + &token,
            3600
        ]))
        .await
        .expect("Database error")
        .unwrap();
}

/// Check if token is in pending registrations in database
///
/// # Arguments
///
/// * `token` - Challenge token
/// * `redis` - Connection to database
pub async fn register_exists(token: &String, redis: &Data<Addr<RedisActor>>) -> Option<String> {
    let res = redis
        .send(Command(resp_array!["GET", "register:".to_owned() + &token]))
        .await
        .expect("Database error")
        .unwrap();

    match res {
        RespValue::BulkString(val) => Some(String::from_utf8(val).unwrap()),
        _ => None,
    }
}

/// Remove a pending registration from the database
///
/// # Arguments
///
/// * `token` - Challenge token
/// * `redis` - Connection to database
pub async fn register_remove(token: &String, redis: &Data<Addr<RedisActor>>) {
    redis
        .send(Command(resp_array!["DEL", "register:".to_owned() + &token]))
        .await
        .expect("Database error")
        .unwrap();
}

/// Adds an user to the database and adds the default settings to the database.
///
/// # Arguments
///
/// * `email` - Email address
/// * `redis` - Connection to database
pub async fn user_add(email: &String, redis: &Data<Addr<RedisActor>>) {
    redis
        .send(Command(resp_array![
            "MSET",
            // User
            format!("user:{}", email),
            "",
            // Temperature
            format!("settings:{}:units:temperature", email),
            "Celsius",
            // Pressure
            format!("settings:{}:units:pressure", email),
            "Bar",
            // Theme
            format!("settings:{}:theme", email),
            "Light",
            // Timeframe
            format!("settings:{}:timeframe", email),
            "Week"
        ]))
        .await
        .expect("Database error")
        .unwrap();
}

/// Retrieves settings from the database for the corresponding user
///
/// # Arguments
///
/// * `email` - Email address
/// * `redis` - Connection to database
///
/// # Remarks
/// Returns an array containing [temperature, pressure, theme, timeframe]
///
/// TODO: Potentially return a struct instead of array.
pub async fn settings_get(email: &String, redis: &Data<Addr<RedisActor>>) -> Vec<String> {
    let res = redis
        .send(Command(resp_array![
            "MGET",
            format!("settings:{}:units:temperature", email),
            format!("settings:{}:units:pressure", email),
            format!("settings:{}:theme", email),
            format!("settings:{}:timeframe", email)
        ]))
        .await
        .expect("Database error")
        .unwrap();

    let res = match res {
        RespValue::Array(val) => Some(val),
        _ => None,
    }
    .unwrap();

    res.iter()
        .filter(|_s| match _s {
            RespValue::BulkString(_s) => true,
            _ => false,
        })
        .map(|s| match s {
            RespValue::BulkString(s) => String::from_utf8(s.to_vec()).unwrap(),
            _ => String::from(""),
        })
        .collect()
}

/// Saves settings for the corresponding user in the database
///
/// # Arguments
///
/// * `email` - Email address
/// * `data` - Data containing settings
/// * `redis` - Connection to database
pub async fn settings_set(
    email: &String,
    data: &settings::SettingsData,
    redis: &Data<Addr<RedisActor>>,
) {
    redis
        .send(Command(resp_array![
            "MSET",
            // Temperature
            format!("settings:{}:units:temperature", email),
            data.temperature.clone(),
            // Pressure
            format!("settings:{}:units:pressure", email),
            data.pressure.clone(),
            // Theme
            format!("settings:{}:theme", email),
            data.theme.clone(),
            // Timeframe
            format!("settings:{}:timeframe", email),
            data.timeframe.clone()
        ]))
        .await
        .expect("Database error")
        .unwrap();
}
