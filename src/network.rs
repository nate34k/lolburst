use reqwest::{Response, Client};

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
    client
        .get(url)
        .send()
        .await
        .expect("Get request failed")
}