use std::{collections::VecDeque, env, io, thread, time::Duration};

use crossbeam::{
    channel::{unbounded, Receiver},
    select,
};
use crossterm::event::{self, Event, KeyCode};
use reqwest::Client;
use serde_json::Value;
use tui::{backend::Backend, widgets::TableState, Terminal};
use tui_logger::{TuiWidgetEvent, TuiWidgetState};

use crate::{
    active_player::{self, AbilityRanks},
    champions::{self, ActiveChampion},
    dmg, network, ui,
    utils::{deserializer, resistance, teams},
};

pub struct App {
    pub burst_table_state: TableState,
    pub burst_table_items: Vec<Vec<String>>,
    pub burst_last: Vec<String>,
    pub logger_state: TuiWidgetState,
    pub draw_logger: bool,
    pub logger_scroll_mode: bool,
    pub gold_last_tick: f64,
    pub gold_total: f64,
    pub gold_per_min: String,
    pub gold_per_min_vecdeque: VecDeque<(f64, f64)>,
    pub gold_per_min_dataset: Vec<(f64, f64)>,
    pub cs_total: f64,
    pub cs_per_min: String,
    pub cs_per_min_vecdeque: VecDeque<(f64, f64)>,
    pub cs_per_min_dataset: Vec<(f64, f64)>,
    pub vs_total: f64,
    pub vs_per_min: String,
    pub vs_per_min_vecdeque: VecDeque<(f64, f64)>,
    pub vs_per_min_dataset: Vec<(f64, f64)>,
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
        let dataset_length = get_dataset_length();
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
            burst_last: vec![
                "0.0".to_string(); 5
            ],
            logger_state: TuiWidgetState::default(),
            draw_logger: false,
            logger_scroll_mode: false,
            gold_last_tick: 500.0,
            gold_total: 0.0,
            gold_per_min: "42".to_string(),
            gold_per_min_vecdeque: VecDeque::from(vec![(0.0, 0.0); dataset_length]),
            gold_per_min_dataset: vec![(0.0, 0.0); dataset_length],
            cs_total: 0.0,
            cs_per_min: "42".to_string(),
            cs_per_min_vecdeque: VecDeque::from(vec![(0.0, 0.0); dataset_length]),
            cs_per_min_dataset: vec![(0.0, 0.0); dataset_length],
            vs_total: 0.0,
            vs_per_min: "42".to_string(),
            vs_per_min_vecdeque: VecDeque::from(vec![(0.0, 0.0); dataset_length]),
            vs_per_min_dataset: vec![(0.0, 0.0); dataset_length],
            use_sample_data: env::var("USE_SAMPLE_DATA").unwrap_or("false".to_string()) == "true",
            active_player_json_url: env::var("ACTIVE_PLAYER_URL").unwrap(),
            active_player_json_sample: env::var("ACTIVE_PLAYER_JSON_SAMPLE").unwrap(),
            all_players_json_url: env::var("ALL_PLAYERS_URL").unwrap(),
            all_players_json_sample: env::var("ALL_PLAYERS_JSON_SAMPLE").unwrap(),
            game_stats_url: env::var("GAME_STATS_URL").unwrap(),
            game_stats_json_sample: env::var("GAME_STATS_JSON_SAMPLE").unwrap(),
        }
    }

    fn on_tick(&mut self, game_time: f64) {
        self.gold_per_min_vecdeque.pop_front();
        self.gold_per_min_vecdeque
            .push_back((game_time.round(), get_per_min(self.gold_total, game_time)));
        self.gold_per_min_dataset = Vec::from(self.gold_per_min_vecdeque.clone());
        self.cs_per_min_vecdeque.pop_front();
        self.cs_per_min_vecdeque
            .push_back((game_time.round(), get_per_min(self.cs_total, game_time)));
        self.cs_per_min_dataset = Vec::from(self.cs_per_min_vecdeque.clone());
        self.vs_per_min_vecdeque.pop_front();
        self.vs_per_min_vecdeque
            .push_back((game_time.round(), get_per_min(self.vs_total, game_time)));
        self.vs_per_min_dataset = Vec::from(self.vs_per_min_vecdeque.clone());
    }
}

pub async fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> io::Result<()> {
    let client: Client = network::build_client().await;

    if app.use_sample_data {
        warn!("use_sample_data is true, using JSON files in resources directory");
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

    let mut cycle: usize = 0;

    let ui_events_rx = setup_ui_events();
    let tick = tick();

    // Applicaiton loop
    loop {
        if app.use_sample_data {
            debug!("cycle: {}", cycle);
            if cycle
                == std::fs::read_dir(&app.active_player_json_sample)
                    .unwrap()
                    .count()
            {
                cycle = 0;
                app.gold_total = 0.0;
                app.gold_last_tick = 500.0;
            }
        }

        let (active_player_data, all_player_data, game_data) =
            deserializer::deserializer(&app, &client, cycle).await;

        if cycle == 0 {
            let offset = env::var("SAMPLE_RATE").unwrap().parse::<usize>().unwrap() / 1000;
            let offset_vec = || -> Vec<(f64, f64)> {
                let mut x = Vec::new();
                for i in 0..get_dataset_length() {
                    x.push(((game_data.game_time - (offset * i) as f64), 0.0));
                }
                x.into_iter().rev().collect()
            };
            app.gold_per_min_vecdeque = VecDeque::from(offset_vec());
            // app.gold_per_min_dataset[ele.0] = ((game_data.game_time - (env::var("SAMPLE_RATE").unwrap().parse::<f64>().unwrap() / 1000.0) * (app.gold_per_min_vecdeque.len() - ele.0) as f64), 0.0);
            app.gold_per_min_dataset = vec![(0.0, 0.0); get_dataset_length()];
            app.cs_per_min_vecdeque = VecDeque::from(offset_vec());
            app.cs_per_min_dataset = vec![(0.0, 0.0); get_dataset_length()];
            app.vs_per_min_vecdeque = VecDeque::from(offset_vec());
            app.vs_per_min_dataset = vec![(0.0, 0.0); get_dataset_length()];
        }

        debug!("game_time: {}", game_data.game_time);

        let opponant_team = teams::OpponantTeam::new(&active_player_data, &all_player_data);

        let resistance =
            resistance::Resistance::new(&active_player_data, &all_player_data, &ddragon_data);

        // TODO: Find a better place for this
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
        app.gold_per_min = format!("{:.1}", get_per_min(app.gold_total, game_data.game_time));

        for i in all_player_data.all_players.iter() {
            if i.summoner_name == active_player_data.summoner_name {
                app.cs_total = i.scores.creep_score as f64;
                app.cs_per_min = format!("{:.1}", get_per_min(app.cs_total, game_data.game_time));
                app.vs_total = i.scores.ward_score as f64;
                app.vs_per_min = format!("{:.1}", get_per_min(app.vs_total, game_data.game_time));
            }
        }

        app.on_tick(game_data.game_time);

        draw(terminal, &mut app);

        // Handle UI events
        loop {
            select! {
                recv(ui_events_rx) -> event => {
                    match event.unwrap() {
                        Event::Key(key_event) => {
                            match key_event.code {
                                KeyCode::Char('q') => {
                                    return Ok(());
                                }
                                KeyCode::Char('s') => {
                                    break;
                                }
                                KeyCode::Char('l') => {
                                    info!("Toggling logger on/off");
                                    app.draw_logger = !app.draw_logger;
                                }
                                KeyCode::PageUp => {
                                    app.logger_state.transition(&TuiWidgetEvent::PrevPageKey);
                                    app.logger_scroll_mode = true;
                                }
                                KeyCode::PageDown => {
                                    app.logger_state.transition(&TuiWidgetEvent::NextPageKey);
                                    app.logger_scroll_mode = true;
                                }
                                KeyCode::Up => {
                                    app.logger_state.transition(&TuiWidgetEvent::UpKey);
                                }
                                KeyCode::Down => {
                                    app.logger_state.transition(&TuiWidgetEvent::DownKey);
                                }
                                KeyCode::Left => {
                                    app.logger_state.transition(&TuiWidgetEvent::LeftKey);
                                }
                                KeyCode::Right => {
                                    app.logger_state.transition(&TuiWidgetEvent::RightKey);
                                }
                                KeyCode::Esc => {
                                    app.logger_state.transition(&TuiWidgetEvent::EscapeKey);
                                    app.logger_scroll_mode = false;
                                }
                                KeyCode::Char(' ') => {
                                    app.logger_state.transition(&TuiWidgetEvent::SpaceKey);
                                }
                                KeyCode::Char('+') => {
                                    app.logger_state.transition(&TuiWidgetEvent::PlusKey);
                                }
                                KeyCode::Char('-') => {
                                    app.logger_state.transition(&TuiWidgetEvent::MinusKey);
                                }
                                KeyCode::Char('h') => {
                                    app.logger_state.transition(&TuiWidgetEvent::HideKey);
                                }
                                KeyCode::Char('f') => {
                                    app.logger_state.transition(&TuiWidgetEvent::FocusKey);
                                }
                                _ => {}
                            }
                            debug!("{:?}", key_event);
                            draw(terminal, &mut app);
                        }
                        Event::Resize(_x, _y) => {
                            draw(terminal, &mut app);
                        }
                        _ => {}
                    }
                }
                recv(tick) -> _ => { break; }
            }
        }

        app.burst_last =  app.burst_table_items.iter().map(|x| x.last().unwrap().clone()).collect::<Vec<_>>();

        cycle += 1;
    }
}

fn draw<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) {
    debug!("Drawing UI");
    terminal
        .draw(|f| {
            let size = f.size();
            ui::ui(f, size, app);
        })
        .unwrap();
}

fn setup_ui_events() -> Receiver<Event> {
    let (tx, rx) = unbounded();
    thread::spawn(move || loop {
        if crossterm::event::poll(Duration::from_millis(
            env::var("SAMPLE_RATE").unwrap().parse::<u64>().unwrap(),
        ))
        .unwrap()
        {
            let event = event::read().unwrap();
            tx.send(event).unwrap();
            if let Event::Key(key_event) = event {
                if let KeyCode::Char('q') = key_event.code {
                    break;
                }
            }
        }
    });

    rx
}

fn tick() -> Receiver<()> {
    let (tx, rx) = unbounded();
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_millis(
                env::var("SAMPLE_RATE").unwrap().parse::<u64>().unwrap(),
            ))
            .await;
            tx.send(()).unwrap();
        }
    });

    rx
}

fn build_enemy_team_display_data(
    champion: &ActiveChampion,
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

fn get_per_min(total: f64, game_time: f64) -> f64 {
    if game_time < 1.0 {
        total.floor() / (game_time / 60.0).ceil()
    } else {
        total.floor() / (game_time / 60.0)
    }
}

pub struct Bounds {
    pub gold: ([f64; 2], [f64; 2]),
    pub gold_labels: ([String; 3], [String; 5]),
    pub cs: ([f64; 2], [f64; 2]),
    pub cs_labels: ([String; 3], [String; 5]),
    pub vs: ([f64; 2], [f64; 2]),
    pub vs_labels: ([String; 3], [String; 5]),
}

impl Bounds {
    pub fn new(app: &App) -> Bounds {
        Bounds {
            gold: (
                [
                    app.gold_per_min_vecdeque.front().unwrap().0,
                    app.gold_per_min_vecdeque.back().unwrap().0,
                ],
                [0.0, 600.0],
            ),
            gold_labels: (
                ["-5:00".to_string(), "-2:30".to_string(), "0:00".to_string()],
                [
                    0.0.to_string(),
                    150.0.to_string(),
                    300.0.to_string(),
                    450.0.to_string(),
                    600.0.to_string(),
                ],
            ),
            cs: (
                [
                    app.cs_per_min_vecdeque.front().unwrap().0,
                    app.cs_per_min_vecdeque.back().unwrap().0,
                ],
                [0.0, 12.0],
            ),
            cs_labels: (
                ["-5:00".to_string(), "-2:30".to_string(), "0:00".to_string()],
                [
                    0.0.to_string(),
                    3.0.to_string(),
                    6.0.to_string(),
                    9.0.to_string(),
                    12.0.to_string(),
                ],
            ),
            vs: (
                [
                    app.vs_per_min_vecdeque.front().unwrap().0,
                    app.vs_per_min_vecdeque.back().unwrap().0,
                ],
                [0.0, 12.0],
            ),
            vs_labels: (
                ["-5:00".to_string(), "-2:30".to_string(), "0:00".to_string()],
                [
                    0.0.to_string(),
                    3.0.to_string(),
                    6.0.to_string(),
                    9.0.to_string(),
                    12.0.to_string(),
                ],
            ),
        }
    }
}

fn get_dataset_length() -> usize {
    (env::var("DATASET_LIFETIME")
        .unwrap()
        .parse::<f64>()
        .unwrap()
        / (env::var("SAMPLE_RATE").unwrap().parse::<f64>().unwrap() / 1000.0)) as usize
}
