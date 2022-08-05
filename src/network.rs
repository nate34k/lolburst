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
    match client.get(url).send().await {
        Ok(res) => res,
        Err(err) => {
            error!("Failed to send Get request to {}: {}", url, err);
            panic!("Failed to send Get request to {}: {}", url, err);
        }
    }
}
