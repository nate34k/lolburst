use std::{io};
use serde_json::{Value};
use tokio::time::{sleep, Duration};
use log::{info, warn, error};
use crate::champions::orianna;
use reqwest::{Response, Client};
use dotenv::dotenv;

pub mod champions;
pub mod dmg;
pub mod active_player;
pub mod all_players;

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

fn calculate_mr(base_mr: f64, mr_per_level: f64, level: i64) -> f64 {
    base_mr + ((level as f64 - 1.0) * mr_per_level)
}

fn get_input(prompt: String) -> String {
    println!("{}", prompt);
    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Error reading input");
    input.trim().to_string()
}

fn get_team(active_player: &active_player::Root, players: &all_players::Root) -> String {
    let mut res = String::new();
    for i in 0..players.all_players.len() {
        let n = players.all_players[i].summoner_name.clone();
        if n == active_player.summoner_name {
            res = players.all_players[i].team.clone();
            break;
        }
    }
    res
}

fn get_opponant_team(active_player: &active_player::Root, players: &all_players::Root) -> Vec<String> {
    let mut opponant_list = Vec::new();
    for i in 0..players.all_players.len() {
        let team = players.all_players[i].team.clone();
        if get_team(active_player, players) != team {
            opponant_list.push(players.all_players[i].champion_name.clone());
        }
    }
    opponant_list
}

async fn request(client: &Client, url: &str) -> Result<Response, reqwest::Error> {
    client.get(url).send().await
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    let active_player_url = String::from("https://127.0.0.1:2999/liveclientdata/activeplayer");
    let active_player_json = String::from("C:/Users/odin/Development/lolburst/src/resources/active_player.json");
    let player_list_url = String::from("​https://127.0.0.1:2999/liveclientdata/playerlist");
    let player_list_json = String::from("C:/Users/odin/Development/lolburst/src/resources/all_players.json");
    let ddragon_url = String::from("http://ddragon.leagueoflegends.com/cdn/12.13.1/data/en_US/champion.json");
    let use_sample_json = true;

    let client = reqwest::Client::builder()
        .danger_accept_invalid_certs(true).build()?;

    let ddragon_data: Value = serde_json::from_str(&request(&client, &ddragon_url).await?.text().await?)?;
    
    let champion = champions::match_champion("Orianna");
    
    loop {
        let active_player_data: active_player::Root;
        let player_data: all_players::Root;
        // Deserialize the JSON data from request
        if use_sample_json {
            active_player_data = serde_json::from_str(&std::fs::read_to_string(&active_player_json)?)?;
            player_data = serde_json::from_str(&std::fs::read_to_string(&player_list_json)?)?;
        }
        else {
            active_player_data = serde_json::from_str(&request(&client, &active_player_url).await?.text().await?)?;
            let player_url_jsonified = String::from("{ \"allPlayers\": ") + &request(&client, &player_list_url).await?.text().await?.to_owned() + "}";
            player_data = serde_json::from_str(&player_url_jsonified)?;
        }

        let opponant_team = get_opponant_team(&active_player_data, &player_data);

        // Set a Vec<f64> for opponant MR values
        let mut mr = Vec::new();
        for i in 0..get_opponant_team(&active_player_data, &player_data).len() {
            mr.push(ddragon_data["data"][opponant_team[i]
                                        .clone()
                                        .replace('\'', "")
                                        .replace(" ", "")]["stats"]["spellblock"].as_f64().unwrap())
        }

        // Other data we need to print
        let ap = active_player_data.champion_stats.ability_power;
        let ability_ranks = AbilityRanks::new(active_player_data.abilities.q.ability_level,
                                                            active_player_data.abilities.w.ability_level,
                                                            active_player_data.abilities.e.ability_level,
                                                            active_player_data.abilities.r.ability_level);

        // Loop to print burst dmg against each enemy champion
        for i in 0..opponant_team.len() {
            println!("Burst is {:.1} vs {}'s {:.0} MR.", calculate_pmd(dmg::calculate_rd(&champion, &ap, &ability_ranks), mr[i]),
                                                         opponant_team[i],
                                                         mr[i]);
        }
                                                        
        println!("================================");
        // Sleep for 5 seconds between running the loop again to save resources
        sleep(Duration::from_secs(15)).await;
    }

    Ok(())
}