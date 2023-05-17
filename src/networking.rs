use reqwest::Client;
use std::{error::Error, io};
use crate::NeonSession;


pub async fn do_http_get(url: String, neon_config: &NeonSession) -> Result<String, Box<dyn Error>> {
    let client = reqwest::Client::builder().build()?;
    let body = client
    .get(url)
    .header("Authorization", format!("Bearer {}", &neon_config.neon_api_key))
    .header("Accept", "application/json")
    .send().await;
    let response = body.expect("Failed to execute request.");
    let body = response.text().await;
    Ok(body.unwrap())
}