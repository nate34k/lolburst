extern crate pretty_env_logger;
#[macro_use] extern crate log;

use std::{env,};
use serde_json::{Value};
use tokio::time::{sleep, Duration};
use log::{info,};
use crate::champions::orianna;
use reqwest::{Client,};
use dotenv;
use crate::dmg::Resistance;
use crate::utils::deserializer;

mod champions;
mod dmg;
mod active_player;
mod all_players;
mod network;
mod utils;

#[derive(Debug)]
pub struct AbilityRanks {
    q_rank: i64,
    w_rank: i64,
    e_rank: i64,
    r_rank: i64,
}

impl AbilityRanks {
    fn new(q_rank: i64, w_rank: i64, e_rank: i64, r_rank: i64) -> Self {
        AbilityRanks { q_rank, w_rank, e_rank, r_rank }
    }
}

// Returns a tuple of the index of the active player in all players and the active players team.
// fn get_team(active_player: &active_player::Root, players: &all_players::Root) -> (usize, String) {
//     let mut res = (0, String::new());
//     for i in 0..players.all_players.len() {
//         let n = players.all_players[i].summoner_name.clone();
//         if n == active_player.summoner_name {
//             res = (i, players.all_players[i].team.clone());
//             break;
//         }
//     }
//     if res.1.is_empty() {
//         panic!("Could not find active player in all players");
//     }
//     res
// }

// struct OpponantTeam {
//     opponant_team: Vec<(String, i64)>,
// }

// impl OpponantTeam {
//     fn new(opponant_team: Vec<(String, i64)>) -> Self {
//         OpponantTeam { opponant_team }
//     }
// }

// fn get_opponant_team(active_player: &active_player::Root, players: &all_players::Root) -> Vec<(String, i64)> {
//     let mut opponant_list = Vec::new();
//     for i in 0..players.all_players.len() {
//         let team = players.all_players[i].team.clone();
//         if get_team(active_player, players).1 != team {
//             opponant_list.push((
//                 players.all_players[i].champion_name
//                     .clone()
//                     .replace('\'', "")
//                     .replace(" ", ""),
//                 players.all_players[i].level));
//         }
//     }
//     opponant_list
// }

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env::set_var("RUST_LOG", "trace");
    dotenv::dotenv().expect("Failed to load env from .env");
    pretty_env_logger::init();

    let active_player_json_locations = deserializer::JSONDataLocations {
        url:  String::from(env::var("ACTIVE_PLAYER_URL")?),
        json: env::var("ACTIVE_PLAYER_JSON")?,
    };

    let all_player_json_locations = deserializer::JSONDataLocations {
        url:  String::from(env::var("ALL_PLAYERS_URL")?),
        json: String::from(env::var("ALL_PLAYERS_JSON")?),
    };

    let client: Client = network::build_client().await;
    
    let deserializer_params = deserializer::DeserializerParams {
        use_sample_json: true,
        active_player_json_locations: active_player_json_locations,
        all_player_json_locations: all_player_json_locations,
        client: &client,
    };

    if deserializer_params.use_sample_json {
        info!("use_sample_json is true. Using JSON files in resources dir.");
    }

    let ddragon_url = "http://ddragon.leagueoflegends.com/cdn/12.13.1/data/en_US/champion.json";

    let ddragon_data: Value = serde_json::from_str(&network::request(&client, &ddragon_url)
        .await
        .text()
        .await
        .expect("Failed to parse data for String"))
        .expect("Failed to deserialize String into JSON Value");
    
    let champion = champions::match_champion("Orianna");
    
    loop {
        
        let (active_player_data, all_player_data) = deserializer::deserializer(&deserializer_params).await;

        let opponant_team = teams::get_opponant_team(&active_player_data, &all_player_data);

        // Set a Vec<f64> for opponant MR values
        let mut mr = Vec::new();
        for i in 0..get_opponant_team(&active_player_data, &all_player_data).len() {
            let champion_name = &opponant_team[i].0;
            let base_mr = ddragon_data["data"][champion_name]["stats"]["spellblock"].as_f64().unwrap();
            let mr_per_level = ddragon_data["data"][champion_name]["stats"]["spellblockperlevel"].as_f64().unwrap();
            let level = opponant_team[i].1 as f64;
            let scaled_mr = base_mr + (mr_per_level * (level - 1.0));
            mr.push(scaled_mr)
        }

        // Set a Vec<f64> for opponant AR values
        let mut ar = Vec::new();
        for i in 0..get_opponant_team(&active_player_data, &all_player_data).len() {
            let champion_name = &opponant_team[i].0;
            let base_mr = ddragon_data["data"][champion_name]["stats"]["armor"].as_f64().unwrap();
            let mr_per_level = ddragon_data["data"][champion_name]["stats"]["armorperlevel"].as_f64().unwrap();
            let level = opponant_team[i].1 as f64;
            let scaled_mr = base_mr + (mr_per_level * (level - 1.0));
            ar.push(scaled_mr)
        }

        // Other data we need to print
        let ability_ranks = AbilityRanks::new(
            active_player_data.abilities.q.ability_level,
            active_player_data.abilities.w.ability_level,
            active_player_data.abilities.e.ability_level,
            active_player_data.abilities.r.ability_level);

        // Loop to print burst dmg against each enemy champion
        for i in 0..opponant_team.len() {
            let resistance = Resistance::new(ar[i], ar[i]);
            println!("Burst is {:.1} vs {}", 
                dmg::burst_dmg(&champion, &active_player_data, &ability_ranks, resistance),
                opponant_team[i].0);
        }
                                                        
        println!("================================");

        // Sleep for 5 seconds between running the loop again to save resources
        sleep(Duration::from_secs(env::var("SAMPLE_RATE")
            .unwrap_or(String::from("15"))
            .parse::<u64>()
            .unwrap_or(15)))
            .await;
    }

    Ok(())
}