use std::{io, thread, time::{Duration, self}};

use crossbeam::{
    channel::{unbounded, Receiver},
    select,
};
use crossterm::event::{self, Event, KeyCode};
use reqwest::Client;
use serde_json::Value;
use slice_deque::SliceDeque;
use tui::{backend::Backend, widgets::TableState, Terminal};
use tui_logger::{TuiWidgetEvent, TuiWidgetState};

use crate::{
    active_player::{self},
    all_players,
    champion::{self, Champion},
    config::Config,
    game_data, network, ui,
    ui::burst_table::BurstTable,
    utils::{deserializer, teams},
};

pub struct App {
    pub burst_table_state: TableState,
    pub burst_table_items: Vec<Vec<String>>,
    pub burst_last: Vec<String>,
    pub logger_state: TuiWidgetState,
    pub draw_logger: bool,
    pub logger_scroll_mode: bool,
    pub gold: ui::gold::Gold,
    pub cs: ui::cs::CS,
    pub vs: ui::vs::VS,
    pub use_sample_data: bool,
    pub active_player_json_url: &'static str,
    pub active_player_json_sample: &'static str,
    pub all_players_json_url: &'static str,
    pub all_players_json_sample: &'static str,
    pub game_stats_url: &'static str,
    pub game_stats_json_sample: &'static str,
}

impl App {
    pub fn new(c: &Config) -> App {
        App {
            burst_table_state: TableState::default(),
            burst_table_items: vec![
                vec!["Lucian".to_string(), "1".to_string(), "0.0".to_string()],
                vec!["Ahri".to_string(), "1".to_string(), "0.0".to_string()],
                vec!["Orianna".to_string(), "1".to_string(), "0.0".to_string()],
                vec!["Diana".to_string(), "1".to_string(), "0.0".to_string()],
                vec!["Blitzcrank".to_string(), "1".to_string(), "0.0".to_string()],
            ],
            burst_last: vec!["0.0".to_string(); 5],
            logger_state: TuiWidgetState::default(),
            draw_logger: false,
            logger_scroll_mode: false,
            gold: ui::gold::Gold::new(),
            cs: ui::cs::CS::new(),
            vs: ui::vs::VS::new(),
            use_sample_data: c.use_sample_data,
            active_player_json_url: crate::ACTIVE_PLAYER_URL,
            active_player_json_sample: crate::ACTIVE_PLAYER_JSON_SAMPLE,
            all_players_json_url: crate::ALL_PLAYERS_URL,
            all_players_json_sample: crate::ALL_PLAYERS_JSON_SAMPLE,
            game_stats_url: crate::GAME_STATS_URL,
            game_stats_json_sample: crate::GAME_STATS_JSON_SAMPLE,
        }
    }

    fn on_tick(&mut self, game_time: f64, cur_gold: f64, cur_cs: i64, cur_vs: f64) {
        self.gold.on_tick(game_time, cur_gold);
        self.cs.on_tick(game_time, cur_cs);
        self.vs.on_tick(game_time, cur_vs);
    }
}

pub struct Data {
    pub active_player_data: active_player::Root,
    pub all_player_data: all_players::Root,
    pub game_data: game_data::Root,
}

pub async fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    mut app: App,
    config: &Config,
) -> io::Result<()> {
    // Build a client
    let client: Client = network::build_client().await;

    // Check if we are using sample data
    if app.use_sample_data {
        warn!("use_sample_data is true, using JSON files in resources directory");
    }

    // Deserialize the Data Dragon into serde_json::Value
    let data_dragon_champions: Value = serde_json::from_str(
        &network::request(&client, crate::DATA_DRAGON_URL)
            .await
            .text()
            .await
            .expect("Failed to parse data for String"),
    )
    .expect("Failed to deserialize String into JSON Value");

    let champion: Champion;
    let player_index: usize;
    let mut cycle: usize = 0;

    // Check if game is ready
    loop {
        let data = deserializer::deserializer(&app, &client, cycle).await;
        match data {
            Ok(data) => {
                let (i, _, c) = teams::get_active_player(&data.active_player_data, &data.all_player_data);
                player_index = i;
                champion = champion::Champion::new(c.as_str());
                break;
            }
            Err(err) => {
                error!("Error: {}", err);
                info!("Retrying in 5 seconds...");
                tokio::time::sleep(Duration::from_secs(5)).await;
                continue;
            }
        }
    }

    // spawn threads to handle additional tasks
    let ui_events_rx = setup_ui_events();

    // Applicaiton loop
    loop {
        let time = time::Instant::now();
        // Check if we are using sample data and if so, check if we need to cycle the data back to the beginning
        if app.use_sample_data {
            debug!("cycle: {}", cycle);
            if cycle
                == std::fs::read_dir(&app.active_player_json_sample)
                    .unwrap()
                    .count()
            {
                cycle = 0;
                app.gold = ui::gold::Gold::new();
            }
        }

        // Deserialize data from Riot API into Data struct
        let data = &deserializer::deserializer(&app, &client, cycle)
            .await
            .unwrap();

        // If app is on the first cycle, reset the datasets
        if cycle == 0 {
            app.gold.reset_datasets(config, data);
            app.cs.reset_datasets(config, data);
            app.vs.reset_datasets(config, data);
        }

        debug!("game_time: {}", data.game_data.game_time);

        // Set burst_table to a new BurstTable
        let burst_table = BurstTable {
            champion: &champion,
            data,
            data_dragon_data: &data_dragon_champions,
            rotation: &config.rotation,
        };

        // Update app.burst_table_items to the correct Vec<Vec<String>>
        app.burst_table_items = BurstTable::build_burst_table_items(burst_table);

        app.on_tick(
            data.game_data.game_time,
            data.active_player_data.current_gold,
            data.all_player_data.all_players[player_index].scores.creep_score,
            data.all_player_data.all_players[player_index].scores.ward_score,
        );

        draw(terminal, &app);

        // Handle UI events
        loop {
            select! {
                recv(ui_events_rx) -> event => {
                    match event.unwrap() {
                        UIEvent::Key(key_event) => {
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
                            draw(terminal, &app);
                        }
                        UIEvent::Resize(_x, _y) => {
                            draw(terminal, &app);
                        }
                    }
                }
                default(Duration::from_secs(config.sample_rate)) => {
                    break;
                }
            }
        }

        app.burst_last = app
            .burst_table_items
            .iter()
            .map(|x| x.last().unwrap().clone())
            .collect::<Vec<_>>();

        cycle += 1;

        info!("cycle took: {:?}", time.elapsed());
    }
}

fn draw<B: Backend>(terminal: &mut Terminal<B>, app: &App) {
    terminal
        .draw(|f| {
            let size = f.size();
            ui::ui(f, size, app);
        })
        .unwrap();
}

enum UIEvent {
    Key(event::KeyEvent),
    Resize(u16, u16),
}

fn setup_ui_events() -> Receiver<UIEvent> {
    let (tx, rx) = unbounded();
    thread::spawn(move || loop {
        let event = event::read().unwrap();
        match event {
            Event::Mouse(_) => {}
            Event::Key(key_code) => {
                tx.send(UIEvent::Key(key_code)).unwrap();
            }
            Event::Resize(x, y) => {
                tx.send(UIEvent::Resize(x, y)).unwrap();
            }
        }
        if let Event::Key(key_event) = event {
            if let KeyCode::Char('q') = key_event.code {
                break;
            }
        }
    });

    rx
}

pub struct Bounds {
    pub gold_labels: ([String; 3], [String; 5]),
    pub cs_labels: ([String; 3], [String; 5]),
    pub vs_labels: ([String; 3], [String; 5]),
}

impl Bounds {
    pub fn new() -> Bounds {
        Bounds {
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
            vs_labels: (
                ["-5:00".to_string(), "-2:30".to_string(), "0:00".to_string()],
                [
                    0.0.to_string(),
                    0.5.to_string(),
                    1.0.to_string(),
                    1.5.to_string(),
                    2.0.to_string(),
                ],
            ),
        }
    }
}

fn get_dataset_length(config: &Config) -> usize {
    (config.dataset_lifetime as f64 / (config.sample_rate as f64)) as usize
}

pub trait Stats {
    fn reset_vecdeque_dataset(&self, config: &Config, data: &Data) -> SliceDeque<(f64, f64)> {
        // Set offset to sample rate and divide by 1000 to get sapmle rate in seconds
        // Offset is used to determine how far back in time the graph should start
        let offset = config.sample_rate as usize;

        // Closure to create a VecDeque with the correct length and values for graphing
        let reversed_vecdeque_with_offset = || -> SliceDeque<(f64, f64)> {
            let mut x = SliceDeque::new();
            for i in 0..get_dataset_length(config) {
                x.push_back(((data.game_data.game_time - (offset * i) as f64), 0.0));
            }
            x.into_iter().rev().collect()
        };

        // Reassign values to the datasets
        reversed_vecdeque_with_offset()
    }

    fn reset_vec_dataset(&self, config: &Config) -> Vec<(f64, f64)> {
        vec![(0.0, 0.0); get_dataset_length(config)]
    }

    fn string_from_per_min(&self) -> String;
}
