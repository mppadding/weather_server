//! Documentation for auth module.
//! Includes authentication and registration.
//!
//! Most functions are called from the `actix-web` framework.
use crate::haak::database;
use crate::haak::email;

use actix::prelude::*;
use actix_redis::RedisActor;
use actix_session::Session;
use actix_web::web::{Data, Json, Query};
use actix_web::HttpResponse;

use rand::rngs::OsRng;
use rand::RngCore;
use serde::{Deserialize, Serialize};

/// Handles HTTP GET requests to `/login`.
/// Displays the login page, redirects to `/` if already logged in.
///
/// # Arguments
///
/// * `session` - Session containing all CookieSession data
///
/// # Remarks
///
/// Should only be called from actix_web
pub async fn login_get(session: Session) -> HttpResponse {
    match session.get::<String>("email").unwrap().is_some() {
        true => {
            return HttpResponse::SeeOther()
                .header(actix_web::http::header::LOCATION, "/")
                .finish()
        }
        false => HttpResponse::Ok().body(include_str!("../../templates/auth/login.html")),
    }
}

/// Identity used in forms
#[derive(Deserialize)]
pub struct Identity {
    email: String,
}

/// LoginChallenge to store in session (user email and challenge token)
#[derive(Serialize, Deserialize, Debug)]
pub struct LoginChallenge {
    email: String,
    challenge: String,
}

/// Handles HTTP POST requests to /login.
/// Validates email (sends 422 UnprocessableEntity if invalid), generates a challenge, stores that
/// challenge in the CookieSession and emails the challenge to the user.
///
/// # Arguments
///
/// * `form` - JSON data of the login form, containing user's email
/// * `session` - Session containing all CookieSession data
/// * `redis` - RedisActor to access redis database
///
/// # Remarks
///
/// Should only be called from actix_web
pub async fn login_submit(
    form: Json<Identity>,
    session: Session,
    redis: Data<Addr<RedisActor>>,
) -> HttpResponse {
    let email = form.email.clone();

    // If logged in -> redirect to /
    if session.get::<String>("email").unwrap().is_some() {
        return HttpResponse::SeeOther()
            .header(actix_web::http::header::LOCATION, "/")
            .finish();
    }

    // If invalid email -> Respond
    if !validator::validate_email(email.as_str()) {
        return HttpResponse::UnprocessableEntity().body("Invalid email");
    }

    // If not in database (user doesnt exist) -> send check email (to prevent getting data)
    if !database::user_exists(&email, &redis).await {
        return HttpResponse::Ok().body("Check your mail for login code");
    }

    let challenge = generate_challenge();

    let _ = session.set(
        "pending_login",
        LoginChallenge {
            email: email.clone(),
            challenge: challenge.clone(),
        },
    );

    match email::send_challenge(email, challenge) {
        Ok(_) => HttpResponse::Ok().body("Check your mail for login code"),
        Err(_) => HttpResponse::InternalServerError().body("Could not send authentication mail"),
    }
}

/// Handles HTTP POST request to /register
/// Sends a registration email to a new user, with verification link
///
/// # Arguments
///
/// * `form` - JSON data of the login form, containing user's email
/// * `session` - Session containing all CookieSession data
/// * `redis` - RedisActor to access redis database
///
/// # Remarks
///
/// Should only be called from actix_web
pub async fn register(
    form: Json<Identity>,
    session: Session,
    redis: Data<Addr<RedisActor>>,
) -> HttpResponse {
    let user = session.get::<String>("email").unwrap();
    let email = form.email.clone();

    // If user is not logged in or not admin -> Unauthorized
    if user.is_none() || !database::user_is_admin(&user.unwrap(), &redis).await {
        return HttpResponse::Unauthorized().finish();
    }

    // If invalid email -> Respond
    if !validator::validate_email(email.as_str()) {
        return HttpResponse::UnprocessableEntity().body("Invalid email");
    }

    if database::user_exists(&email, &redis).await {
        return HttpResponse::UnprocessableEntity().body("Email already registered");
    }

    let challenge = generate_challenge();

    database::register_email(&email, &challenge, &redis).await;

    match email::send_register(email, challenge) {
        Ok(_) => HttpResponse::Ok().body("Check your mail for login code"),
        Err(_) => HttpResponse::InternalServerError().body("Could not send authentication mail"),
    }
}

/// Handles HTTP GET request to /logout
/// Logs the user out if they are logged in and redirects them to /login
///
/// # Arguments
///
/// * `session` - Session containing all CookieSession data
///
/// # Remarks
///
/// Should only be called from actix_web
pub async fn logout(session: Session) -> HttpResponse {
    let user = session.get::<String>("email").unwrap();

    if user.is_some() {
        session.purge();
    }

    HttpResponse::SeeOther()
        .header(actix_web::http::header::LOCATION, "/login")
        .finish()
}

/// Handles HTTP GET requests to /poll_login. Returns 200 OK if logged in and otherwise 406
/// NotAcceptable
///
/// # Arguments
///
/// * `session` - Session containing all CookieSession data
///
/// # Remarks
///
/// Should only be called from actix_web
pub async fn poll_login(session: Session) -> HttpResponse {
    match session.get::<String>("email").unwrap().is_some() {
        true => HttpResponse::Ok().body(""),
        false => HttpResponse::NotAcceptable().body(""),
    }
}

/// Creates a new 32 byte challenge to use in login/registration
fn generate_challenge() -> String {
    let mut challenge = vec![0u8; 32];
    OsRng.fill_bytes(&mut challenge);
    base64::encode_config(&challenge, base64::URL_SAFE)
}

/// Query data of verify_login call (remaps ?c -> challenge)
#[derive(Deserialize)]
pub struct VerifyQuery {
    #[serde(rename = "c")]
    challenge: String,
}

/// Handles HTTP GET requests to /verify_login
///
/// # Arguments
///
/// * `query` - Query containing the challenge token
/// * `session` - Session containing all CookieSession data
///
/// # Remarks
///
/// Should only be called from actix_web
pub async fn verify_login(Query(query): Query<VerifyQuery>, session: Session) -> HttpResponse {
    let pending_login: Option<LoginChallenge> = session
        .get::<LoginChallenge>("pending_login")
        .unwrap_or(None);

    let login_challenge = match pending_login {
        Some(lc) => lc,
        None => {
            return HttpResponse::SeeOther()
                .header(actix_web::http::header::LOCATION, "/login")
                .finish()
        }
    };

    if login_challenge.challenge == query.challenge {
        let _ = session.set("email", login_challenge.email);

        HttpResponse::Ok().body(include_str!("../../templates/auth/verified.html"))
    } else {
        HttpResponse::Unauthorized().body(include_str!("../../templates/auth/invalid_token.html"))
    }
}

/// Handles HTTP GET requests to /verify_register
/// Registers the user and displays a link to login
///
/// # Arguments
///
/// * `query` - Query containing the challenge token
/// * `redis` - RedisActor to access redis database
///
/// # Remarks
///
/// Should only be called from actix_web
pub async fn verify_register(
    Query(query): Query<VerifyQuery>,
    redis: Data<Addr<RedisActor>>,
) -> HttpResponse {
    let email = database::register_exists(&query.challenge, &redis).await;

    if email.is_none() {
        return HttpResponse::Unauthorized()
            .body(include_str!("../../templates/auth/invalid_token.html"));
    }

    match email {
        Some(e) => {
            database::user_add(&e, &redis).await;
            database::register_remove(&query.challenge, &redis).await;
            HttpResponse::Ok().body(include_str!("../../templates/auth/registered.html"))
        }
        None => HttpResponse::Unauthorized()
            .body(include_str!("../../templates/auth/invalid_token.html")),
    }
}
