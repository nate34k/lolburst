use std::time::Duration;

use reqwest::{Client, Response};

pub async fn build_client() -> Client {
    let client = reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .build()
        .expect("Failed to build client");
    info!("Client built");
    client
}

pub async fn request(client: &Client, url: &str) -> Response {
    info!("Sending Get request to {}", url);
    loop {
        match client.get(url).send().await {
            Ok(res) => return res,
            Err(err) => {
                error!("Didn't receive a response from {}: {}", url, err);
                std::thread::sleep(Duration::from_secs(5));
                continue;
            }
        }
    }
}
