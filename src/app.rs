use std::{env, io, time::Duration};

use crossterm::event::{self, Event, KeyCode};
use reqwest::Client;
use serde_json::Value;
use tui::{backend::Backend, widgets::TableState, Terminal};

use crate::{
    active_player::{self, AbilityRanks},
    champions::{self, ActiveChampion},
    dmg, network, ui,
    utils::{deserializer, resistance, teams},
};

pub struct App {
    pub burst_table_state: TableState,
    pub burst_table_items: Vec<Vec<String>>,
    pub gold_per_min: String,
    pub cs_per_min: String,
    pub vs_per_min: String,
    pub use_sample_data: bool,
    pub active_player_json_url: String,
    pub active_player_json_sample: String,
    pub all_players_json_url: String,
    pub all_players_json_sample: String,
}

impl App {
    pub fn new() -> App {
        App {
            burst_table_state: TableState::default(),
            burst_table_items: vec![
                vec![
                    "Row11".to_string(),
                    "Row12".to_string(),
                    "Row13".to_string(),
                ],
                vec![
                    "Row21".to_string(),
                    "Row22".to_string(),
                    "Row23".to_string(),
                ],
                vec![
                    "Row31".to_string(),
                    "Row32".to_string(),
                    "Row33".to_string(),
                ],
                vec![
                    "Row41".to_string(),
                    "Row42".to_string(),
                    "Row43".to_string(),
                ],
                vec![
                    "Row51".to_string(),
                    "Row52".to_string(),
                    "Row53".to_string(),
                ],
            ],
            gold_per_min: "42".to_string(),
            cs_per_min: "42".to_string(),
            vs_per_min: "42".to_string(),
            use_sample_data: env::var("USE_SAMPLE_DATA").unwrap_or("false".to_string()) == "true",
            active_player_json_url: env::var("ACTIVE_PLAYER_URL").unwrap(),
            active_player_json_sample: env::var("ACTIVE_PLAYER_JSON_SAMPLE").unwrap(),
            all_players_json_url: env::var("ALL_PLAYERS_URL").unwrap(),
            all_players_json_sample: env::var("ALL_PLAYERS_JSON_SAMPLE").unwrap(),
        }
    }
}

pub async fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> io::Result<()> {
    let client: Client = network::build_client().await;

    if app.use_sample_data {
        info!("use_sample_data is true, using JSON files in resources directory");
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

    // Applicaiton loop
    loop {
        let (active_player_data, all_player_data) = deserializer::deserializer(&app, &client).await;

        let opponant_team = teams::OpponantTeam::new(&active_player_data, &all_player_data);

        let resistance =
            resistance::Resistance::new(&active_player_data, &all_player_data, &ddragon_data);

        // Other data we need to print
        let ability_ranks = AbilityRanks::new(
            active_player_data.abilities.q.ability_level,
            active_player_data.abilities.w.ability_level,
            active_player_data.abilities.e.ability_level,
            active_player_data.abilities.r.ability_level,
        );

        app.burst_table_items = build_enemy_team_display_data(
            &champion,
            active_player_data,
            ability_ranks,
            opponant_team,
            resistance,
        );

        info!("Drawing UI");
        terminal.draw(|mut f| {
            let size = f.size();
            ui::ui(&mut f, size, &mut app);
        })?;

        if crossterm::event::poll(Duration::from_millis(
            env::var("SAMPLE_RATE").unwrap().parse::<u64>().unwrap(),
        ))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::F(12) => return Ok(()),
                    _ => {}
                }
            }
        }
    }
}

fn build_enemy_team_display_data<'a>(
    champion: &'a ActiveChampion,
    active_player_data: active_player::Root,
    ability_ranks: AbilityRanks,
    opponant_team: teams::OpponantTeam,
    resistance: resistance::Resistance,
) -> Vec<Vec<String>> {
    let mut ret = Vec::new();
    // Loop to print burst dmg against each enemy champion
    for i in 0..opponant_team.opponants.len() {
        let mut row = Vec::new();
        let r = dmg::Resistance::new(resistance.armor[i], resistance.magic_resist[i]);
        let burst_dmg = dmg::burst_dmg(&champion, &active_player_data, &ability_ranks, r);
        row.push(opponant_team.opponants[i].0.clone());
        row.push(opponant_team.opponants[i].1.to_string());
        row.push(burst_dmg.floor().to_string());
        ret.push(row);
    }
    ret
}
