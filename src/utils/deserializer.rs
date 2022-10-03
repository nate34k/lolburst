use crate::{
    app::{App},
    data::LiveGame,
    network,
};
use reqwest::Client;
use std::fs;

const ALL_DATA_JSON_SAMPLE: &str = "./resources/all_data/all_data_";
const ALL_DATA_URL: &str = "https://127.0.0.1:2999/liveclientdata/allgamedata";

pub async fn deserialize(
    app: &App,
    client: &Client,
    cycle: usize,
) -> Result<LiveGame, serde_json::Error> {
    let data: Result<LiveGame, serde_json::Error>;

    if app.use_sample_data {
        let p = String::from(ALL_DATA_JSON_SAMPLE);
        data = serde_json::from_str(
            &fs::read_to_string(p + &cycle.to_string() + ".json").expect("Failed to read file"),
        )
    } else {
        data = serde_json::from_str(
            &network::request(client, ALL_DATA_URL)
                .await
                .text()
                .await
                .expect("Failed to parse data for String"),
        );
    }

    data
}
