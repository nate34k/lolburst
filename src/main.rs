extern crate pretty_env_logger;
#[macro_use] extern crate log;

use std::{io, env, fs};
use serde::__private::de;
use serde_json::{Value};
use tokio::time::{sleep, Duration};
use log::{info, warn, error};
use crate::champions::orianna;
use reqwest::{Response, Client};
use dotenv;

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

fn get_opponant_team(active_player: &active_player::Root, players: &all_players::Root) -> Vec<(String, i64)> {
    let mut opponant_list = Vec::new();
    for i in 0..players.all_players.len() {
        let team = players.all_players[i].team.clone();
        if get_team(active_player, players) != team {
            opponant_list.push((
                players.all_players[i].champion_name
                    .clone()
                    .replace('\'', "")
                    .replace(" ", ""),
                players.all_players[i].level));
        }
    }
    opponant_list
}

async fn request(client: &Client, url: &str) -> Response {
    info!("Sending Get request to {}", url);
    client
        .get(url)
        .send()
        .await
        .expect("Get request failed")
}

struct JSONDataLocations {
    url: String,
    json: String,
}

struct DeserializerParams<'a> {
    use_sample_json: bool,
    active_player_json_locations: JSONDataLocations,
    all_player_json_locations: JSONDataLocations,
    client: &'a Client
}

async fn deserializer(derserializer_params: &DeserializerParams<'_>) -> (active_player::Root, all_players::Root) {
    let active_player_data: active_player::Root;
    let all_player_data: all_players::Root;
    let use_sample_json = derserializer_params.use_sample_json;
    let active_player_json_locations = &derserializer_params.active_player_json_locations;
    let all_player_json_locations = &derserializer_params.all_player_json_locations;
    let client = derserializer_params.client;

    if use_sample_json {
        info!("use_sample_json is true. Using JSON files in resources dir.");

        active_player_data = serde_json::from_str(&fs::read_to_string(&active_player_json_locations.json)
            .expect("Failed to read string from file"))
            .expect("Failed to deserialize string to active_player::Root");
        all_player_data = serde_json::from_str(&fs::read_to_string(&all_player_json_locations.json)
            .expect("Failed to read string from file"))
            .expect("Failed to deserialize string into all_players::Root");
    } else {
        active_player_data = serde_json::from_str(&request(&client, &active_player_json_locations.url)
            .await
            .text()
            .await
            .expect("Failed to parse data for String"))
            .expect("Failed to deserialize String into active_player::Root");
        let player_url_jsonified = String::from("{ \"allPlayers\": ") + &request(&client, &all_player_json_locations.url).await.text().await.expect("msg").to_owned() + "}";
        all_player_data = serde_json::from_str(&player_url_jsonified).expect("msg");
    }

    (active_player_data, all_player_data)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env::set_var("RUST_LOG", "trace");
    dotenv::dotenv().expect("Failed to load env from .env");
    pretty_env_logger::init();

    let active_player_json_locations = JSONDataLocations {
        url:  String::from(env::var("ACTIVE_PLAYER_URL")?),
        json: String::from(env::var("ACTIVE_PLAYER_JSON")?),
    };

    let all_player_json_locations = JSONDataLocations {
        url:  String::from(env::var("ALL_PLAYERS_URL")?),
        json: String::from(env::var("ALL_PLAYERS_JSON")?),
    };

    let client: Client = reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .build()
        .expect("Failed to build client");
        info!("Client built!");

    let deserializer_params = DeserializerParams {
        use_sample_json: true,
        active_player_json_locations: active_player_json_locations,
        all_player_json_locations: all_player_json_locations,
        client: &client,
    };

    //let active_player_url = "https://127.0.0.1:2999/liveclientdata/activeplayer";
    //let active_player_json = "C:/Users/odin/Development/lolburst/src/resources/active_player.json";
    //let player_list_url = "​https://127.0.0.1:2999/liveclientdata/playerlist";
    //let player_list_json = "C:/Users/odin/Development/lolburst/src/resources/all_players.json";
    //let use_sample_json = true;
    let ddragon_url = "http://ddragon.leagueoflegends.com/cdn/12.13.1/data/en_US/champion.json";

    let ddragon_data: Value = serde_json::from_str(&request(&client, &ddragon_url)
        .await
        .text()
        .await
        .expect("Failed to parse data for String"))
        .expect("Failed to deserialize String into JSON Value");
    
    let champion = champions::match_champion("Orianna");
    
    loop {
        let all_data = deserializer(&deserializer_params).await;
        let active_player_data: active_player::Root = all_data.0;
        let all_player_data: all_players::Root = all_data.1;

        // Deserialize the JSON data
        //if use_sample_json {
        //    info!("use_sample_json is true. Using JSON files in resources dir.");
        //    active_player_data = serde_json::from_str(&fs::read_to_string(&active_player_json)
        //        .expect("Failed to read string from file"))?;
        //    player_data = serde_json::from_str(&fs::read_to_string(&player_list_json)?)?;
        //} else {
        //    active_player_data = serde_json::from_str(&request(&client, &active_player_url)
        //        .await
        //        .text()
        //        .await
        //        .expect("Failed to parse data for String"))
        //        .expect("Failed to deserialize String into active_player::Root");
        //    let player_url_jsonified = String::from("{ \"allPlayers\": ") + &request(&client, &player_list_url).await.text().await?.to_owned() + "}";
        //    player_data = serde_json::from_str(&player_url_jsonified)?;
        //}


        let opponant_team = get_opponant_team(&active_player_data, &all_player_data);

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

        // Other data we need to print
        let ap = active_player_data.champion_stats.ability_power;
        let ability_ranks = AbilityRanks::new(
            active_player_data.abilities.q.ability_level,
            active_player_data.abilities.w.ability_level,
            active_player_data.abilities.e.ability_level,
            active_player_data.abilities.r.ability_level);

        // Loop to print burst dmg against each enemy champion
        for i in 0..opponant_team.len() {
            println!("Burst is {:.1} vs {}'s {:.0} MR.", 
                calculate_pmd(dmg::calculate_rd(&champion, &ap, &ability_ranks), mr[i]),
                opponant_team[i].0,
                mr[i]);
        }
                                                        
        println!("================================");
        // Sleep for 5 seconds between running the loop again to save resources
        sleep(Duration::from_secs(15)).await;
    }

    Ok(())
}