extern crate pretty_env_logger;
#[macro_use]
extern crate log;

use crate::champions::orianna;
use crate::utils::{deserializer, resistance, teams};
use champions::ActiveChampion;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use log::info;
use reqwest::Client;
use serde_json::Value;
use std::env;
use std::io;
use tokio::time::Duration;
use tui::layout::{Direction, Rect};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Layout},
    style::{Color, Style},
    widgets::{Block, Borders, Cell, Row, Table, TableState},
    Frame, Terminal,
};
use tui_logger::{TuiLoggerLevelOutput, TuiLoggerWidget};
use utils::teams::OpponantTeam;

mod active_player;
mod all_players;
mod app;
mod champions;
mod dmg;
mod network;
mod utils;

#[derive(Debug)]
pub struct AbilityRanks {
    q_rank: i64,
    w_rank: i64,
    e_rank: i64,
    r_rank: i64,
}

impl AbilityRanks {
    fn new(q_rank: i64, w_rank: i64, e_rank: i64, r_rank: i64) -> Self {
        AbilityRanks {
            q_rank,
            w_rank,
            e_rank,
            r_rank,
        }
    }
}

fn build_enemy_team_display_data<'a>(
    champion: &'a ActiveChampion,
    active_player_data: active_player::Root,
    ability_ranks: AbilityRanks,
    opponant_team: OpponantTeam,
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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // env::set_var("RUST_LOG", "trace");
    dotenv::dotenv().expect("Failed to load env from .env");

    // Early initialization of the logger

    // Set max_log_level to Trace
    tui_logger::init_logger(log::LevelFilter::Trace).unwrap();

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

fn ui<B: Backend>(f: &mut Frame<B>, size: Rect, app: &mut app::App) {
    let block = Block::default().borders(Borders::ALL);
    let inner_area = block.inner(size);
    f.render_widget(block, size);
    let constraints = vec![
        Constraint::Length(13),
        Constraint::Percentage(100),
        Constraint::Min(15),
    ];
    let rects = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(inner_area);

    let normal_style = Style::default().bg(Color::Blue);
    let header_cells = ["Champion", "Level", "Burst"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(Color::DarkGray)));
    let header = Row::new(header_cells)
        .style(normal_style)
        .height(1)
        .bottom_margin(1);
    let rows = app.items.iter().map(|item| {
        let height = item
            .iter()
            .map(|content| content.chars().filter(|c| *c == '\n').count())
            .max()
            .unwrap_or(0)
            + 1;
        let cells = item.iter().map(|c| Cell::from(c.as_str()));
        Row::new(cells).height(height as u16).bottom_margin(1)
    });
    let t = Table::new(rows)
        .header(header)
        .block(Block::default().borders(Borders::ALL).title("lolburst"))
        .widths(&[
            Constraint::Length(12),
            Constraint::Length(5),
            Constraint::Length(5),
        ]);
    f.render_stateful_widget(t, rects[0], &mut app.state);
    let tui_w: TuiLoggerWidget = TuiLoggerWidget::default()
        .block(
            Block::default()
                .title("Log")
                .border_style(Style::default().fg(Color::White).bg(Color::Black))
                .borders(Borders::ALL),
        )
        .output_separator('|')
        .output_timestamp(Some("%F %H:%M:%S%.3f".to_string()))
        .output_level(Some(TuiLoggerLevelOutput::Long))
        .output_target(false)
        .output_file(false)
        .output_line(false)
        .style_error(Style::default().fg(Color::Red))
        .style_debug(Style::default().fg(Color::Green))
        .style_warn(Style::default().fg(Color::Yellow))
        .style_trace(Style::default().fg(Color::Magenta))
        .style_info(Style::default().fg(Color::Cyan));
    f.render_widget(tui_w, rects[1]);
}
