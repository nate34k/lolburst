use std::{
    io, thread,
    time::{self, Duration, Instant},
};

use crossbeam::{
    channel::{unbounded, Receiver},
    select,
};
use crossterm::event::{self, Event, KeyCode};
use reqwest::Client;
use serde_json::Value;
use slice_deque::SliceDeque;
use tui::{backend::Backend, widgets::TableState, Terminal};
use tui_logger::TuiWidgetState;

use crate::{
    active_player::{self},
    all_players,
    champion::{self, Champion},
    config::Config,
    game_data,
    handlers::keyboard::{handle_keyboard, KeyboardHandler},
    network, ui,
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
    last_tick: time::Instant,
    pub use_sample_data: bool,
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
            last_tick: time::Instant::now(),
            use_sample_data: c.use_sample_data,
        }
    }

    fn on_tick(&mut self, game_time: f64, cur_gold: f64, cur_cs: i64, cur_vs: f64) {
        self.gold.on_tick(game_time, cur_gold);
        self.cs.on_tick(game_time, cur_cs);
        self.vs.on_tick(game_time, cur_vs);
    }

    fn check_elapsed_time(&mut self, sample_rate: u64) {
        if self.last_tick.elapsed() >= Duration::from_millis(sample_rate * 1000) {
            self.last_tick = Instant::now();
        }
    }

    fn get_timeout(&self, sample_rate: u64) -> Duration {
        Duration::from_millis(sample_rate * 1000)
            .checked_sub(self.last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_millis(0))
    }

    fn reset_datasets(&mut self, config: &Config, game_time: f64) {
        self.gold.reset_dataset(config, game_time);
        self.cs.reset_dataset(config, game_time);
        self.vs.reset_dataset(config, game_time);
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
                match data.http_status {
                    Some(_) => {
                        warn!("Warning, game not ready");
                        info!("Retrying in 5 seconds...");
                        tokio::time::sleep(Duration::from_secs(1)).await;
                        cycle += 1;
                        continue;
                    }
                    None => {
                        let (i, _, c) = teams::get_active_player(
                            &data.active_player.unwrap(),
                            &data.all_players.unwrap(),
                        );
                        player_index = i;
                        champion = champion::Champion::new(c.as_str());
                        app.reset_datasets(config, data.game_data.unwrap().game_time);
                        break;
                    }
                }
            }
            Err(err) => {
                error!("Error: {}", err);
                info!("Retrying in 5 seconds...");
                tokio::time::sleep(Duration::from_secs(5)).await;
                continue;
            }
        }
    }

    // Spawn threads to handle additional tasks
    let ui_events_rx = setup_ui_events();

    // Applicaiton loop
    loop {
        let time = time::Instant::now();

        draw(terminal, &app);

        app.burst_last = app
            .burst_table_items
            .iter()
            .map(|x| x.last().unwrap().clone())
            .collect::<Vec<_>>();

        let timeout = app.get_timeout(config.sample_rate);

        // Handle UI events
        select! {
            recv(ui_events_rx) -> event => {
                match handle_keyboard(event.unwrap(), &mut app) {
                    KeyboardHandler::Quit => { break; },
                    KeyboardHandler::None => {},
                }
            }
            default(timeout) => {
                // Check if we are using sample data and if so, check if we need to cycle the data back to the beginning
                if app.use_sample_data {
                    debug!("cycle: {}", cycle);
                    if cycle
                        == 6000
                    {
                        cycle = 0;
                        app.gold = ui::gold::Gold::new();
                        app.cs = ui::cs::CS::new();
                        app.vs = ui::vs::VS::new();
                    }
                }

                // Deserialize data from Riot API into Data struct
                let data = &deserializer::deserializer(&app, &client, cycle)
                    .await
                    .unwrap();

                info!("CS: {}", data.all_players.as_ref().unwrap()[player_index].scores.creep_score);

                // If app is on the first cycle, reset the datasets
                if cycle == 0 {
                    app.reset_datasets(config, data.game_data.as_ref().unwrap().game_time);
                }

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
                    data.game_data.as_ref().unwrap().game_time,
                    data.active_player.as_ref().unwrap().current_gold,
                    data.all_players.as_ref().unwrap()[player_index]
                        .scores
                        .creep_score,
                    data.all_players.as_ref().unwrap()[player_index]
                        .scores
                        .ward_score,
                );

                cycle += 1;
            }
        }

        app.check_elapsed_time(config.sample_rate);

        info!("cycle took: {:?}", time.elapsed());
    }
    Ok(())
}

fn draw<B: Backend>(terminal: &mut Terminal<B>, app: &App) {
    terminal
        .draw(|f| {
            let size = f.size();
            ui::ui(f, size, app);
        })
        .unwrap();
}

pub enum UIEvent {
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

fn get_dataset_length(config: &Config) -> usize {
    (config.dataset_lifetime as f64 / (config.sample_rate as f64)) as usize
}

pub trait Stats {
    fn reset_vecdeque_dataset(&self, config: &Config, game_time: f64) -> SliceDeque<(f64, f64)> {
        // Set offset to sample rate and divide by 1000 to get sapmle rate in seconds
        // Offset is used to determine how far back in time the graph should start
        let offset = config.sample_rate as usize;

        // Closure to create a VecDeque with the correct length and values for graphing
        let reversed_vecdeque_with_offset = || -> SliceDeque<(f64, f64)> {
            let mut x = SliceDeque::new();
            for i in 0..get_dataset_length(config) {
                x.push_back(((game_time - (offset * i) as f64), 0.0));
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
