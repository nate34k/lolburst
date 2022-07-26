use std::{io, thread};
use std::sync::{Arc, Mutex};
use serde_json::{Value};
use tokio::time::{sleep, Duration};
use log::{info, warn, error};
use crate::champions::orianna;
use device_query::{DeviceEvents, DeviceState, Keycode, device_state};

pub mod champions;
pub mod dmg;

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

fn calculate_ignite(level: i32) -> f64 {
    50.0 + f64::from(level * 20)
}

fn calculate_pmd(rd: f64, mr: f64) -> f64 {
    let pmd = rd / (1.0 + (mr/100.0));
    pmd
}

fn get_input(prompt: String) -> String {
    println!("{}", prompt);
    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Error reading input");
    input.trim().to_string()
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let url = String::from("https://static.developer.riotgames.com/docs/lol/liveclientdata_sample.json");
    let client = reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .build()?;

    let champion = champions::match_champion("Orianna");
    let mut mr = 0.0;

    let device_state = DeviceState::new();
    let _guard = device_state.on_key_down(move|key| {
        mr += 0.0;
    });
    loop {
        let request = match client.get(&url).send().await {
            Ok(request) => request,
            Err(err) => {
                error!("Error sending get request: {}", err);
                return Err(err.into());
            }
        };
        let game_data: Value = match serde_json::from_str(&request.text().await?) {
            Ok(game_data) => game_data,
            Err(err) => {
                error!("Error parsing game data: {}", err);
                break;
            }
        };
        let ap = game_data["activePlayer"]["championStats"]["abilityPower"].as_f64().unwrap();
        let ability_ranks = AbilityRanks::new(game_data["activePlayer"]["abilities"]["Q"]["abilityLevel"].as_i64().unwrap(),
                                                            game_data["activePlayer"]["abilities"]["W"]["abilityLevel"].as_i64().unwrap(),
                                                            game_data["activePlayer"]["abilities"]["E"]["abilityLevel"].as_i64().unwrap(),
                                                            game_data["activePlayer"]["abilities"]["R"]["abilityLevel"].as_i64().unwrap());

        println!("{}'s Burst is {:.1} vs {:.0} MR.", game_data["activePlayer"]["summonerName"].as_str().unwrap().replace('"', ""),
                                                     calculate_pmd(dmg::calculate_rd(&champion, &ap, &ability_ranks), mr),
                                                     mr);

        sleep(Duration::from_secs(5)).await;
    }

    Ok(())
}