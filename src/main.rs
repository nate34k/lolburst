#[macro_use]
extern crate log;

use active_player::AbilityRanks;
use chrono::{DateTime, Utc};
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::io;
use tui::{backend::CrosstermBackend, Terminal};

mod active_player;
mod all_players;
mod app;
mod champion;
mod config;
mod data;
mod dmg;
mod game_data;
mod handlers;
mod network;
mod ui;
mod utils;

const DATA_DRAGON_URL: &str =
    "http://ddragon.leagueoflegends.com/cdn/12.13.1/data/en_US/champion.json";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Set dt to DateTime<Local> for logging
    let dt = chrono::Utc::now();

    // Init logger
    init_logger(&dt);

    // Load config
    let config = &config::setup_config();

    // Setup terminal
    let mut terminal = setup_terminal()?;

    println!("Loading data...");

    // Initialize app
    // Create app
    let app = app::App::new(config);
    // Run app
    let res = app::run_app(&mut terminal, app, config).await;

    info!("Cleaning up terminal");

    // Restore terminal
    restore_terminal(&mut terminal)?;

    tui_logger::move_events();
    info!("Exiting");

    // If app::run_app errors print error
    if let Err(err) = res {
        println!("{:?}", err)
    }

    Ok(())
}

fn init_logger(dt: &DateTime<Utc>) {
    // Create log file
    let log_file = handlers::dir::create_log_file(&dt).unwrap();
    // Set log file
    tui_logger::set_log_file(log_file.as_str()).unwrap();
    // Set max_log_level to Trace
    tui_logger::init_logger(log::LevelFilter::Debug).unwrap();
    // Set default level for unknown targets to Trace
    tui_logger::set_default_level(log::LevelFilter::Trace);
}

fn setup_terminal() -> Result<Terminal<CrosstermBackend<io::Stdout>>, std::io::Error> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    Terminal::new(backend)
}

fn restore_terminal(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
) -> Result<(), std::io::Error> {
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    Ok(())
}
