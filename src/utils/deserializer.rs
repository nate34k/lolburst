use crate::{active_player, all_players, app::App, game_data, network};
use reqwest::Client;
use std::fs;

pub async fn deserializer(
    app: &App,
    client: &Client,
    cycle: usize,
) -> (active_player::Root, all_players::Root, game_data::Root) {
    let active_player_data: active_player::Root;
    let all_player_data: all_players::Root;
    let game_data: game_data::Root;

    if app.use_sample_data {
        let p = String::from(&app.active_player_json_sample);
        active_player_data = serde_json::from_str(
            &fs::read_to_string(p + &format!("/active_player_{}.json", cycle))
                .expect("Failed to read string from file"),
        )
        .expect("Failed to deserialize string to active_player::Root");
        let p = String::from(&app.all_players_json_sample);
        let all_players_jsonified = String::from("{ \"allPlayers\": ")
            + &fs::read_to_string(p + &format!("_{}.json", cycle))
                .expect("Failed to read string from file")
            + "}";
        all_player_data = serde_json::from_str(&all_players_jsonified)
            .expect("Failed to deserialize string into all_players::Root");
        let p = String::from(&app.game_stats_json_sample);
        game_data = serde_json::from_str(
            &fs::read_to_string(p + &format!("_{}.json", cycle))
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
