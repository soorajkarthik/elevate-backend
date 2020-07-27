use crate::models::alerts::{Alert, TokenDistanceWrapper};

pub fn send_alert_notification (alert: &Alert, user_token_distances: Vec<TokenDistanceWrapper>) -> usize {
    println!("{:#?}", alert);
    println!("{:#?}", user_token_distances);
    usize::MAX
}