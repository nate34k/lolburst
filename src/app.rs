use std::{collections::VecDeque, env, io, time::Duration};

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
    pub gold_last_tick: f64,
    pub gold_total: f64,
    pub gold_per_min: String,
    pub gold_per_min_past_20: VecDeque<(f64, f64)>,
    pub gold_per_min_arr: [(f64, f64); 20],
    pub cs_per_min: String,
    pub vs_per_min: String,
    pub use_sample_data: bool,
    pub active_player_json_url: String,
    pub active_player_json_sample: String,
    pub all_players_json_url: String,
    pub all_players_json_sample: String,
    pub game_stats_url: String,
    pub game_stats_json_sample: String,
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
            gold_last_tick: 0.0,
            gold_total: 0.0,
            gold_per_min: "42".to_string(),
            gold_per_min_past_20: VecDeque::from(vec![(0.0, 0.0); 20]),
            gold_per_min_arr: [(0.0, 0.0); 20],
            cs_per_min: "42".to_string(),
            vs_per_min: "42".to_string(),
            use_sample_data: env::var("USE_SAMPLE_DATA").unwrap_or("false".to_string()) == "true",
            active_player_json_url: env::var("ACTIVE_PLAYER_URL").unwrap(),
            active_player_json_sample: env::var("ACTIVE_PLAYER_JSON_SAMPLE").unwrap(),
            all_players_json_url: env::var("ALL_PLAYERS_URL").unwrap(),
            all_players_json_sample: env::var("ALL_PLAYERS_JSON_SAMPLE").unwrap(),
            game_stats_url: env::var("GAME_STATS_URL").unwrap(),
            game_stats_json_sample: env::var("GAME_STATS_JSON_SAMPLE").unwrap(),
        }
    }

    fn on_tick(&mut self, gold_total: f64, game_time: f64) {
        self.gold_per_min_past_20.pop_front();
        self.gold_per_min_past_20
            .push_back((game_time.round(), get_gold_per_min(gold_total, game_time)));
        self.gold_per_min_past_20
            .iter()
            .clone()
            .enumerate()
            .for_each(|(i, g)| self.gold_per_min_arr[i] = (g.0, g.1));
    }

    pub fn get_gold_x_bounds(&self) -> [f64; 2] {
        [
            self.gold_per_min_past_20.front().unwrap().0,
            self.gold_per_min_past_20.back().unwrap().0,
        ]
    }

    pub fn get_gold_x_bounds_labels(&self) -> [String; 3] {
        [
            format!("{}", self.gold_per_min_past_20.front().unwrap().0),
            format!(
                "{}",
                ((self.gold_per_min_past_20.back().unwrap().0)
                    - self.gold_per_min_past_20.front().unwrap().0)
                    / 2.0
            ),
            format!("{}", self.gold_per_min_past_20.back().unwrap().0),
        ]
    }

    pub fn get_gold_y_bounds(&self) -> [f64; 2] {
        [
            self.gold_per_min_past_20.front().unwrap().1 * 0.8,
            self.gold_per_min_past_20.back().unwrap().1 * 1.2,
        ]
    }

    pub fn get_y_bounds_labels(&self) -> [String; 3] {
        [
            format!("{:.0}", self.gold_per_min_past_20.front().unwrap().1 * 0.8),
            format!("{:.0}", self.gold_per_min_past_20.back().unwrap().1),
            format!("{:.0}", self.gold_per_min_past_20.back().unwrap().1 * 1.2),
        ]
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
        let (active_player_data, all_player_data, game_data) =
            deserializer::deserializer(&app, &client).await;

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
            &active_player_data,
            ability_ranks,
            opponant_team,
            resistance,
        );

        app.gold_total = get_total_gold_earned(
            &active_player_data.current_gold,
            &app.gold_last_tick,
            &app.gold_total,
        );
        app.gold_last_tick = active_player_data.current_gold;
        app.on_tick(app.gold_total, game_data.game_time);
        app.gold_per_min = format!("{:.1}", get_gold_per_min(app.gold_total, game_data.game_time));
        info!("x_bounds: {:?}", app.get_gold_x_bounds());
        info!("y_bounds: {:?}", app.get_gold_y_bounds());
        info!("{:?}", &app.gold_per_min_arr);

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
    active_player_data: &active_player::Root,
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

fn get_total_gold_earned(current_gold: &f64, gold_last_tick: &f64, gold_total: &f64) -> f64 {
    if current_gold <= gold_last_tick {
        *gold_total
    } else {
        (current_gold - gold_last_tick) + gold_total
    }
}

fn get_gold_per_min(gold_total: f64, game_time: f64) -> f64 {
    if game_time < 1.0 {
        gold_total.floor() / (game_time / 60.0).ceil()
    } else {
        gold_total.floor() / (game_time / 60.0)
    }
}
