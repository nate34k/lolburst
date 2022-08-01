extern crate pretty_env_logger;
#[macro_use]
extern crate log;

use crate::champions::orianna;
use crate::utils::{deserializer, resistance, teams};
use log::info;
use reqwest::Client;
use serde_json::Value;
use std::env;
use tokio::time::{sleep, Duration};

mod active_player;
mod all_players;
mod champions;
mod dmg;
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
        AbilityRanks {
            q_rank,
            w_rank,
            e_rank,
            r_rank,
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env::set_var("RUST_LOG", "trace");
    dotenv::dotenv().expect("Failed to load env from .env");
    pretty_env_logger::init();

    let active_player_json_locations = deserializer::JSONDataLocations {
        url: env::var("ACTIVE_PLAYER_URL")?,
        json: env::var("ACTIVE_PLAYER_JSON")?,
    };

    let all_player_json_locations = deserializer::JSONDataLocations {
        url: env::var("ALL_PLAYERS_URL")?,
        json: env::var("ALL_PLAYERS_JSON")?,
    };

    let client: Client = network::build_client().await;

    let deserializer_params = deserializer::DeserializerParams {
        use_sample_json: true,
        active_player_json_locations,
        all_player_json_locations,
        client: &client,
    };

    if deserializer_params.use_sample_json {
        info!("use_sample_json is true. Using JSON files in resources dir.");
    }

    let ddragon_url = "http://ddragon.leagueoflegends.com/cdn/12.13.1/data/en_US/champion.json";

    let ddragon_data: Value = serde_json::from_str(
        &network::request(&client, ddragon_url)
            .await
            .text()
            .await
            .expect("Failed to parse data for String"),
    )
    .expect("Failed to deserialize String into JSON Value");

    let champion = champions::match_champion("Orianna");

    loop {
        let (active_player_data, all_player_data) =
            deserializer::deserializer(&deserializer_params).await;

        let opponant_team = teams::OpponantTeam::new(&active_player_data, &all_player_data);

        let resistance =
            resistance::Resistance::new(&active_player_data, &all_player_data, &ddragon_data);

        // Set a Vec<f64> for opponant AR values
        let mut ar = Vec::new();
        for i in 0..opponant_team.opponants.len() {
            let champion_name = &opponant_team.opponants[i].0;
            let base_mr = ddragon_data["data"][champion_name]["stats"]["armor"]
                .as_f64()
                .unwrap();
            let mr_per_level = ddragon_data["data"][champion_name]["stats"]["armorperlevel"]
                .as_f64()
                .unwrap();
            let level = opponant_team.opponants[i].1 as f64;
            let scaled_mr = base_mr + (mr_per_level * (level - 1.0));
            ar.push(scaled_mr)
        }

        // Other data we need to print
        let ability_ranks = AbilityRanks::new(
            active_player_data.abilities.q.ability_level,
            active_player_data.abilities.w.ability_level,
            active_player_data.abilities.e.ability_level,
            active_player_data.abilities.r.ability_level,
        );

        // Loop to print burst dmg against each enemy champion
        for i in 0..opponant_team.opponants.len() {
            let r = dmg::Resistance::new(resistance.armor[i], resistance.magic_resist[i]);
            println!(
                "Burst is {:.1} vs {}",
                dmg::burst_dmg(&champion, &active_player_data, &ability_ranks, r),
                opponant_team.opponants[i].0
            );
        }

        println!("================================");

        // Sleep for 5 seconds between running the loop again to save resources
        sleep(Duration::from_secs(
            env::var("SAMPLE_RATE")
                .unwrap_or_else(|_| String::from("15"))
                .parse::<u64>()
                .unwrap_or(15),
        ))
        .await;
    }

    Ok(())
}
