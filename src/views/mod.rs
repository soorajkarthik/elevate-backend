use lettre::smtp::authentication::IntoCredentials;
use lettre::{SmtpClient, Transport};
use lettre_email::EmailBuilder;
use std::env;

pub mod alert;
pub mod catchers;
pub mod location;
pub mod pages;
pub mod request;
pub mod user;

pub fn send_email(to: String, subject: String, message: String) -> Result<String, String> {
    let smtp_server = env::var("SMTP_SERVER").unwrap();
    let username = env::var("NO_REPLY_EMAIL").unwrap();
    let password = env::var("NO_REPLY_PASSWORD").unwrap();
    let email = match EmailBuilder::new()
        .to(to.as_str())
        .from(username.clone())
        .subject(subject.as_str())
        .html(message.as_str())
        .build()
    {
        Ok(email) => email.into(),
        Err(_) => return Err(String::from("Couldn't generate email")),
    };

    let credentials = (username, password).into_credentials();
    let mut client = match SmtpClient::new_simple(smtp_server.as_str()) {
        Ok(client) => client.credentials(credentials).transport(),
        Err(_) => return Err(String::from("Couldn't connect to smtp client")),
    };

    match client.send(email) {
        Ok(_) => Ok(String::from("Email sent successfully")),
        Err(_) => Err(String::from("Couldn't send reset email")),
    }
}

#[macro_export]
macro_rules! send_email_using_file {
    ($to:expr, $subject:expr, $file_path:expr, $alt_message:expr) => {
        match send_email(
            String::from($to),
            String::from($subject),
            read_to_string($file_path).unwrap_or_else(|_| String::from($alt_message)),
        ) {
            Ok(_) => info!("Email sent successfully"),
            Err(_) => info!("Error sending email"),
        }
    };

    ($to:expr, $subject:expr, $file_path:expr, $alt_message:expr, $format_replacement:expr) => {
        match send_email(
            String::from($to),
            String::from($subject),
            read_to_string($file_path)
                .unwrap_or_else(|_| String::from($alt_message))
                .replace("{}", $format_replacement),
        ) {
            Ok(_) => info!("Email sent successfully"),
            Err(_) => info!("Error sending email"),
        }
    };
}
