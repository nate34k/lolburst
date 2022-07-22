use std::{vec, io};
use serde_json::{Value};

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

struct AbilityRanks {
    qrank: usize,
    wrank: usize,
    erank: usize,
    rrank: usize,
}

impl AbilityRanks {
    fn new(qrank: usize, wrank: usize, erank: usize, rrank: usize) -> Self {
        AbilityRanks { qrank, wrank, erank, rrank }
    }
}

fn abilityranks_builder() -> AbilityRanks {
    AbilityRanks::new(get_input(String::from("Enter Q Rank")).parse::<usize>().unwrap(),
                      get_input(String::from("Enter W Rank")).parse::<usize>().unwrap(),
                      get_input(String::from("Enter E Rank")).parse::<usize>().unwrap(),
                      get_input(String::from("Enter R Rank")).parse::<usize>().unwrap())
}

fn orianna_bulder() -> Orianna {
    Orianna::new(String::from("Orianna"),
                 vec![60.0,90.0,120.0,150.0,180.0,0.5],
                 vec![60.0,105.0,150.0,195.0,240.0,0.8],
                 vec![60.0,90.0,120.0,150.0,180.0,0.3],
                 vec![200.0,275.0,350.0,0.8],)
}

fn calculate_rd(ap: f64, abilityranks: AbilityRanks) -> f64 {
    let orianna = orianna_bulder();
    let qrank = abilityranks.qrank;
    let wrank = abilityranks.wrank;
    let erank = abilityranks.erank;
    let rrank = abilityranks.rrank;
    let qrd = (orianna.qdmg[qrank-1]) + (orianna.qdmg[5] * ap);
    let wrd = (orianna.wdmg[wrank-1]) + (orianna.wdmg[5] * ap);
    let erd = (orianna.edmg[erank-1]) + (orianna.edmg[5] * ap);
    let rrd = (orianna.rdmg[rrank-1]) + (orianna.rdmg[3] * ap);
    qrd + wrd + erd + rrd
}

fn calculate_ignite(level: i32) -> f64 {
    50.0 + f64::from(level * 20)
}

fn calculate_pmd(rd: f64, mr: f64) -> f64 {
    let pmd = rd / (1.0 + (mr/100.0));
    pmd.floor() + calculate_ignite(6)
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
    let orianna = orianna_bulder();

    let ap = get_input(String::from("Enter AP")).parse::<f64>().unwrap();
    let mr = get_input(String::from("Enter MR")).parse::<f64>().unwrap();

    println!("{}'s Burst is {:?} vs {:?} MR.", orianna.name, calculate_pmd(calculate_rd(ap, abilityranks_builder()), mr), mr);

    let request = reqwest::get("https://static.developer.riotgames.com/docs/lol/liveclientdata_sample.json")
        .await?
        .text()
        .await?;

    let game_data: Value = serde_json::from_str(&request)?;

    println!("{}", game_data["activePlayer"]["abilities"]["E"]["abilityLevel"]);

    Ok(())
}