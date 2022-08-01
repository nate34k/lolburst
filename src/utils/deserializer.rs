use crate::{active_player, all_players, network};
use reqwest::Client;
use std::fs;

pub struct JSONDataLocations {
    pub url: String,
    pub json: String,
}

pub struct DeserializerParams<'a> {
    pub use_sample_json: bool,
    pub active_player_json_locations: JSONDataLocations,
    pub all_player_json_locations: JSONDataLocations,
    pub client: &'a Client,
}

pub async fn deserializer(
    derserializer_params: &DeserializerParams<'_>,
) -> (active_player::Root, all_players::Root) {
    let active_player_data: active_player::Root;
    let all_player_data: all_players::Root;
    let use_sample_json = derserializer_params.use_sample_json;
    let active_player_json_locations = &derserializer_params.active_player_json_locations;
    let all_player_json_locations = &derserializer_params.all_player_json_locations;
    let client = derserializer_params.client;

    if use_sample_json {
        active_player_data = serde_json::from_str(
            &fs::read_to_string(&active_player_json_locations.json)
                .expect("Failed to read string from file"),
        )
        .expect("Failed to deserialize string to active_player::Root");
        all_player_data = serde_json::from_str(
            &fs::read_to_string(&all_player_json_locations.json)
                .expect("Failed to read string from file"),
        )
        .expect("Failed to deserialize string into all_players::Root");
    } else {
        active_player_data = serde_json::from_str(
            &network::request(client, &active_player_json_locations.url)
                .await
                .text()
                .await
                .expect("Failed to parse data for String"),
        )
        .expect("Failed to deserialize String into active_player::Root");
        let player_url_jsonified = String::from("{ \"allPlayers\": ")
            + &network::request(client, &all_player_json_locations.url)
                .await
                .text()
                .await
                .expect("msg")
            + "}";
        all_player_data = serde_json::from_str(&player_url_jsonified).expect("msg");
    }

    (active_player_data, all_player_data)
}
