use crate::{
    app::{self, App},
    data::LiveGame,
    network,
};
use reqwest::Client;
use std::fs;

pub const ACTIVE_PLAYER_JSON_SAMPLE: &str = "./resources/active_player";
const ACTIVE_PLAYER_URL: &str = "https://127.0.0.1:2999/liveclientdata/activeplayer";
const ALL_PLAYERS_JSON_SAMPLE: &str = "./resources/all_players/all_players";
const ALL_PLAYERS_URL: &str = "https://127.0.0.1:2999/liveclientdata/playerlist";
const GAME_STATS_JSON_SAMPLE: &str = "./resources/game_data/game_data";
const GAME_STATS_URL: &str = "https://127.0.0.1:2999/liveclientdata/gamestats";
const ALL_DATA_JSON_SAMPLE: &str = "./resources/all_data/all_data_";

// #[derive(Debug, serde::Deserialize)]
// pub struct Deserialize {}

// impl Deserialize {
//     pub fn from_str<'a, T>(s: &'a str) -> Result<T, serde_json::Error>
//     where T: serde::de::Deserialize<'a>
//     {
//         match serde_json::from_str(s) {
//             Ok(v) => Ok(v),
//             Err(e) => Err(e),
//         }
//     }

// }

pub async fn deserializer(
    app: &App,
    client: &Client,
    cycle: usize,
) -> Result<LiveGame, serde_json::Error> {
    let data: Result<LiveGame, serde_json::Error>;
    // let all_player_data: Result<_, serde_json::Error>;
    // let game_data: Result<_, serde_json::Error>;

    if app.use_sample_data {
        let p = String::from(ALL_DATA_JSON_SAMPLE);
        data = serde_json::from_str(
            &fs::read_to_string(p + &cycle.to_string() + ".json").expect("Failed to read file"),
        )
        // let p = String::from(ALL_PLAYERS_JSON_SAMPLE);
        // let all_players_jsonified = String::from("{ \"allPlayers\": ")
        //     + &fs::read_to_string(p + &format!("_{}.json", cycle))
        //         .expect("Failed to read string from file")
        //     + "}";
        // all_player_data = serde_json::from_str(&all_players_jsonified);
        // let p = String::from(GAME_STATS_JSON_SAMPLE);
        // game_data = serde_json::from_str(
        //     &fs::read_to_string(p + &format!("_{}.json", cycle))
        //         .expect("Failed to read string from file"),
        // );
    } else {
        data = serde_json::from_str(
            &network::request(client, ACTIVE_PLAYER_URL)
                .await
                .text()
                .await
                .expect("Failed to parse data for String"),
        );
        // let player_url_jsonified = String::from("{ \"allPlayers\": ")
        //     + &network::request(client, ALL_PLAYERS_URL)
        //         .await
        //         .text()
        //         .await
        //         .expect("msg")
        //     + "}";
        // all_player_data = serde_json::from_str(&player_url_jsonified);
        // game_data = serde_json::from_str(
        //     &network::request(client, GAME_STATS_URL)
        //         .await
        //         .text()
        //         .await
        //         .expect("Failed to parse data for String"),
        // );
    }

    data

    // if active_player_data.is_err() || all_player_data.is_err() || game_data.is_err() {
    //     return Err(Box::new(std::io::Error::new(
    //         std::io::ErrorKind::InvalidData,
    //         "Failed to deserialize data",
    //     )));
    // }

    // Ok(app::Data {
    //     active_player_data: active_player_data.unwrap(),
    //     all_player_data: all_player_data.unwrap(),
    //     game_data: game_data.unwrap(),
    // })
}
