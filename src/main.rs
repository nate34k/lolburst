extern crate pretty_env_logger;
#[macro_use]
extern crate log;

use crate::champions::orianna;
use active_player::AbilityRanks;
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
mod champions;
mod dmg;
mod network;
mod ui;
mod utils;
mod game_data;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // env::set_var("RUST_LOG", "trace");
    dotenv::dotenv().expect("Failed to load env from .env");

    // Early initialization of the logger

    // Set max_log_level to Trace
    tui_logger::init_logger(log::LevelFilter::Debug).unwrap();

    // Set default level for unknown targets to Trace
    tui_logger::set_default_level(log::LevelFilter::Trace);

    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // create app and run it
    let app = app::App::new();
    let res = app::run_app(&mut terminal, app).await;

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err)
    }

    Ok(())
}
