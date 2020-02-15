//! Main file
mod haak;

#[macro_use]
extern crate redis_async;

use std::env;

use actix_files::{Files, NamedFile};
use actix_redis::{RedisActor, RedisSession};
use actix_web::{middleware, web, App, HttpRequest, HttpServer, Result};

use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};

/// Favicon handler
/// Loads the favicon in ./templates/favicon.ico
///
/// # Remarks
///
/// Should be called from actix_web
async fn favicon(_req: HttpRequest) -> Result<NamedFile> {
    Ok(NamedFile::open("./templates/favicon.ico")?)
}

/// Main function.
///
/// Gets cookie secret from environment, setups redis, the logger and routes.
#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_web=info,actix_redis=info");
    env_logger::init();

    // Cookie secret is used to encrypt the session token
    let cookie_secret = base64::decode(
        &env::var("COOKIE_SECRET_KEY")
        .expect("Cookie secret key not set, generate a new one with export COOKIE_SECRET_KEY=`cat /dev/urandom | head -c 32 | base64`")
    ).unwrap();

    let port =
        &env::var("WEATHER_PORT").expect("Port not set, set it with export WEATHER_PORT=443");

    &env::var("WEATHER_URL").expect("URL not set, set it with export WEATHER_URL=<url>");

    let ip = &env::var("WEATHER_IP").expect("IP not set, set it with export WEATHER_IP=<ip>");

    let mut builder = SslAcceptor::mozilla_intermediate(SslMethod::tls()).unwrap();
    builder
        .set_private_key_file("key.pem", SslFiletype::PEM)
        .unwrap();
    builder.set_certificate_chain_file("cert.pem").unwrap();

    HttpServer::new(move || {
        App::new()
            // redis session middleware
            .data(RedisActor::start("127.0.0.1:6379"))
            .wrap(RedisSession::new("127.0.0.1:6379", &cookie_secret[..]))
            // enable logger
            .wrap(middleware::Logger::default())
            // Resources
            .service(Files::new(
                "/resources/images",
                "./templates/resources/images/",
            ))
            .service(Files::new(
                "/resources/scripts",
                "./templates/resources/scripts/",
            ))
            .service(Files::new(
                "/resources/styles",
                "./templates/resources/styles/",
            ))
            .route("/favicon.ico", web::get().to(favicon))
            // Debug
            //.service(web::resource("/test").route(web::get().to(test)))
            // Authentication
            .service(
                web::resource("/login")
                    .route(web::get().to(haak::auth::login_get))
                    .route(web::post().to(haak::auth::login_submit)),
            )
            .service(web::resource("/poll_login").to(haak::auth::poll_login))
            .service(web::resource("/verify_login").to(haak::auth::verify_login))
            .service(web::resource("/logout").to(haak::auth::logout))
            .service(web::resource("/register").to(haak::auth::register))
            .service(web::resource("/verify_register").to(haak::auth::verify_register))
            // Settings
            .service(
                web::resource("/settings")
                    .route(web::get().to(haak::settings::settings_index))
                    .route(web::post().to(haak::settings::settings_save)),
            )
            // Graphs
            .service(web::resource("/").to(haak::graph::graph_index))
    })
    .bind_openssl(format!("{}:{}", ip, port), builder)?
    .run()
    .await
}
