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
mod game_data;
mod network;
mod ui;
mod utils;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    
    // Load .env file
    dotenv::dotenv().expect("Failed to load env from .env");

    // Early initialization of the logger
    // Set max_log_level to Trace
    tui_logger::init_logger(log::LevelFilter::Trace).unwrap();
    // Set default level for unknown targets to Trace
    tui_logger::set_default_level(log::LevelFilter::Trace);

    // Setup terminal
    let mut terminal = setup_terminal()?;

    // Initialize app
    // Create app
    let app = app::App::new();
    // Run app
    let res = app::run_app(&mut terminal, app).await;

    // Restore terminal
    // disable_raw_mode()?;
    // execute!(
    //     terminal.backend_mut(),
    //     LeaveAlternateScreen,
    //     DisableMouseCapture
    // )?;
    // terminal.show_cursor()?;

    // Restore terminal
    restore_terminal(&mut terminal)?;

    // If app::run_app errors print error
    if let Err(err) = res {
        println!("{:?}", err)
    }

    Ok(())
}

fn setup_terminal() -> Result<Terminal<CrosstermBackend<io::Stdout>>, std::io::Error> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend);
    terminal
}

fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<(), std::io::Error> {
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    Ok(())
}
