use std::{vec, io};
use serde_json::{Value};

pub mod network;

#[derive(Debug)]
struct Orianna {
    name: String,
    qdmg: Vec<f64>,
    wdmg: Vec<f64>,
    edmg: Vec<f64>,
    rdmg: Vec<f64>,
}

impl Orianna {
    fn new(name: String, qdmg: Vec<f64>, wdmg: Vec<f64>, edmg: Vec<f64>, rdmg: Vec<f64>) -> Self {
        Orianna { name, qdmg, wdmg, edmg, rdmg }
    }
}

#[derive(Debug)]
struct AbilityRanks {
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

fn orianna_builder() -> Orianna {
    Orianna::new(String::from("Orianna"),
                 vec![0.0,60.0,90.0,120.0,150.0,180.0,0.5],
                 vec![0.0,60.0,105.0,150.0,195.0,240.0,0.7],
                 vec![0.0,60.0,90.0,120.0,150.0,180.0,0.3],
                 vec![0.0,200.0,275.0,350.0,0.8],)
}

fn calculate_rd(ap: f64, abilityranks: &AbilityRanks) -> f64 {
    let orianna = orianna_builder();
    let qrank = abilityranks.q_rank;
    let wrank = abilityranks.w_rank;
    let erank = abilityranks.e_rank;
    let rrank = abilityranks.r_rank;
    let qrd = (orianna.qdmg[qrank as usize]) + (orianna.qdmg[6] * ap);
    let wrd = (orianna.wdmg[wrank as usize]) + (orianna.wdmg[6] * ap);
    let erd = (orianna.edmg[erank as usize]) + (orianna.edmg[6] * ap);
    let rrd = (orianna.rdmg[rrank as usize]) + (orianna.rdmg[4] * ap);
    qrd + wrd
}

fn calculate_ignite(level: i32) -> f64 {
    50.0 + f64::from(level * 20)
}

fn calculate_pmd(rd: f64, mr: f64) -> f64 {
    let pmd = rd / (1.0 + (mr/100.0));
    pmd + calculate_ignite(18)
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
    let url = String::from("https://127.0.0.1:2999/liveclientdata/allgamedata");
    let client = reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .build()?;

    let orianna = orianna_builder();


    
    loop {
        let request = network::get_request(client, url).await;
        let game_data: Value = serde_json::from_str(&request?.text().await?)?;
        let ap = game_data["activePlayer"]["championStats"]["abilityPower"].as_f64().unwrap();
        let mr = 50.0;
        let ability_ranks = AbilityRanks::new(game_data["activePlayer"]["abilities"]["Q"]["abilityLevel"].as_i64().unwrap(),
                                                            game_data["activePlayer"]["abilities"]["W"]["abilityLevel"].as_i64().unwrap(),
                                                            game_data["activePlayer"]["abilities"]["E"]["abilityLevel"].as_i64().unwrap(),
                                                            game_data["activePlayer"]["abilities"]["R"]["abilityLevel"].as_i64().unwrap());
        println!("{}'s Burst is {:.1} vs {:.0} MR.", orianna.name, calculate_pmd(calculate_rd(ap, &ability_ranks), mr), mr);
    }

    Ok(())
}