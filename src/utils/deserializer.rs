use crate::{active_player, all_players, app::App, network, game_data};
use reqwest::Client;
use std::fs;

// pub struct DeserializerParams<'a> {
//     pub use_sample_json: App,
//     pub active_player_json_url: App,
//     pub active_player_json_sample: App,
//     pub all_players_json_url: App,
//     pub all_players_json_sample: App,
//     pub client: &'a Client,
// }

// impl<'a> DeserializerParams<'a> {
//     pub fn new(
//         use_sample_json: App,
//         active_player_json_url: App,
//         active_player_json_sample: App,
//         all_players_json_url: App,
//         all_players_json_sample: App,
//         client: &'a Client,
//     ) -> Self {
//         DeserializerParams {
//             use_sample_json,
//             active_player_json_url,
//             active_player_json_sample,
//             all_players_json_url,
//             all_players_json_sample,
//             client: &client,
//         }
//     }
// }

pub async fn deserializer(app: &App, client: &Client) -> (active_player::Root, all_players::Root, game_data::Root) {
    let active_player_data: active_player::Root;
    let all_player_data: all_players::Root;
    let game_data: game_data::Root;

    if app.use_sample_data {
        active_player_data = serde_json::from_str(
            &fs::read_to_string(&app.active_player_json_sample)
                .expect("Failed to read string from file"),
        )
        .expect("Failed to deserialize string to active_player::Root");
        all_player_data = serde_json::from_str(
            &fs::read_to_string(&app.all_players_json_sample)
                .expect("Failed to read string from file"),
        )
        .expect("Failed to deserialize string into all_players::Root");
        game_data = serde_json::from_str(
            &fs::read_to_string(&app.game_stats_json_sample)
                .expect("Failed to read string from file"),
        )
        .expect("Failed to deserialize string into game_data::Root");
    } else {
        active_player_data = serde_json::from_str(
            &network::request(client, &app.active_player_json_url)
                .await
                .text()
                .await
                .expect("Failed to parse data for String"),
        )
        .expect("Failed to deserialize String into active_player::Root");
        let player_url_jsonified = String::from("{ \"allPlayers\": ")
        + &network::request(client, &app.all_players_json_url)
        .await
        .text()
        .await
        .expect("msg")
        + "}";
        all_player_data = serde_json::from_str(&player_url_jsonified).expect("msg");
        game_data = serde_json::from_str(
            &network::request(client, &app.game_stats_url)
                .await
                .text()
                .await
                .expect("Failed to parse data for String"),
        )
        .expect("Failed to deserialize String into game_data::Root");
    }

    (active_player_data, all_player_data, game_data)
}
