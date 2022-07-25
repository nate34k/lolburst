use std::{vec, io};
use std::collections::HashMap;
use reqwest::Response;
use serde_json::{Value};
use tokio::time::{sleep, Duration};
use log::{info, warn, error};
use crate::champions::orianna;

pub mod champions;

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
            return Err(err.into());
        }
    };

    let champion = crate::champions::ActiveChampion::match_champion("Orianna");

    loop {
        let request = match client.get(&url).send().await {
            Ok(request) => request,
            Err(err) => {
                println!("Error sending get request: {}", err);
                return Err(err.into());
            }
        };
        let game_data: Value = match serde_json::from_str(&request.text().await?) {
            Ok(game_data) => game_data,
            Err(err) => {
                println!("Error parsing game data: {}", err);
                break;
            }
        };
        let ap = game_data["activePlayer"]["championStats"]["abilityPower"].as_f64().unwrap();
        let mr = 50.0;
        let ability_ranks = AbilityRanks::new(game_data["activePlayer"]["abilities"]["Q"]["abilityLevel"].as_i64().unwrap(),
                                                            game_data["activePlayer"]["abilities"]["W"]["abilityLevel"].as_i64().unwrap(),
                                                            game_data["activePlayer"]["abilities"]["E"]["abilityLevel"].as_i64().unwrap(),
                                                            game_data["activePlayer"]["abilities"]["R"]["abilityLevel"].as_i64().unwrap());

        println!("{}'s Burst is {:.1} vs {:.0} MR.", game_data["activePlayer"]["summonderName"],
                                                     calculate_pmd(champion(&champion::orianna, &ap, &ability_ranks), mr),
                                                     mr);

        println!("{:?}", champion);
        sleep(Duration::from_secs(5)).await;
    }

    Ok(())
}