//! Documentation for email sending
//!
//! # Examples
//! ```
//! match send_challenge("test@test.com", "generated_challenge") {
//!     Ok() => {
//!         // Handle success
//!     },
//!     Err(_) => {
//!         // Handle error
//!     }
//! }
//! ```

use lettre::sendmail::error::Error;
use lettre::sendmail::SendmailTransport;
use lettre::Transport;
use lettre_email::EmailBuilder;

use std::env;

/// Sends a register email to an user
/// Returns `Ok` on success or `Err` on failure
///
/// # Arguments
///
/// * `recipient` - Email address of user
/// * `code` - Challenge token
///
/// # Examples
/// ```
/// match send_register("test@test.com", "generated_challenge") {
///     Ok() => {
///         // Handle success
///     },
///     Err(_) => {
///         // Handle error
///     }
/// }
/// ```
///
/// # Remarks
/// Email should be validated. This function **does not** validate the email input
pub fn send_register(recipient: String, code: String) -> Result<(), Error> {
    let weather_url = &env::var("WEATHER_URL").unwrap();
    let email = EmailBuilder::new()
        .to(recipient)
        .from(format!("weather@{}", weather_url))
        .subject("Weather Station Registration")
        .html(format!("Hello,<br /><br />Your Weather Station Admin has generated a registration request for you.<br />Press the following link to register for the web interface. <a href=\"https://{}/verify_register?c={}\">Register.</a><br /><br />HAAK Weather Station", weather_url, code))
        .build()
        .unwrap();

    let mut sender = SendmailTransport::new();
    sender.send(email.into())
}

/// Sends login challenge email to user
/// Returns `Ok` on success or `Err` on failure
///
/// # Arguments
///
/// * `recipient` - Email address of user
/// * `code` - Challenge token
///
/// # Examples
/// ```
/// match send_challenge("test@test.com", "generated_challenge") {
///     Ok() => {
///         // Handle success
///     },
///     Err(_) => {
///         // Handle error
///     }
/// }
/// ```
///
/// # Remarks
/// Email should be validated. This function **does not** validate the email input
pub fn send_challenge(recipient: String, code: String) -> Result<(), Error> {
    let weather_url = &env::var("WEATHER_URL").unwrap();
    let email = EmailBuilder::new()
        .to(recipient)
        .from(format!("weather@{}", weather_url))
        .subject("Weather Station Login Attempt")
        .html(format!("Hello,<br /><br />You are receiving this email because a login has been requested for the Weather Station.<br />Press the following link to authorize the request. <a href=\"https://{}/verify_login?c={}\">Authorize Request.</a><br /><br />HAAK Weather Station", weather_url, code))
        .build()
        .unwrap();

    let mut sender = SendmailTransport::new();
    sender.send(email.into())
}
