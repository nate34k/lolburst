use reqwest::{Client, Response};

pub async fn get_request(client: Client, url: String) -> Result<Response, reqwest::Error> {
    client.get(url).send().await
}