use std::error::Error;
use crate::NeonSession;
use std::collections::HashMap;

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

pub async fn do_http_post(url:String, postbody: &HashMap<String,String>, neon_config: &NeonSession) -> Result<String, Box<dyn Error>> {
    let client = reqwest::Client::builder().build()?;
    let body = client
    .post(url)
    .json(&postbody)
    .header("Authorization", format!("Bearer {}", &neon_config.neon_api_key))
    .header("Accept", "application/json")
    .send().await;
    let response = body.expect("Failed to execute request.");
    let body = response.text().await;
    Ok(body.unwrap())
}

pub async fn do_http_delete(url: String, neon_config: &NeonSession) -> Result<String, Box<dyn Error>> {
    let client = reqwest::Client::builder().build()?;
    println!("url: {}", url);
    let body = client
    .delete(url)
    .header("Authorization", format!("Bearer {}", &neon_config.neon_api_key))
    .header("Accept", "application/json")
    .send().await;
    let response = body.expect("Failed to execute request.");
    let body = response.text().await;
    Ok(body.unwrap())
}