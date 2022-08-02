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
use tui_logger::{TuiLoggerWidget, TuiLoggerLevelOutput};
use std::env;
use std::{error::Error, io};
use tokio::time::{sleep, Duration};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Row, Table, TableState},
    Frame, Terminal,
};
use utils::teams::OpponantTeam;

mod active_player;
mod all_players;
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

fn display_data<'a>(
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
    env::set_var("RUST_LOG", "trace");
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
    let app = App::new();
    let res = run_app(&mut terminal, app).await;

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

struct App {
    state: TableState,
    items: Vec<Vec<String>>,
}

impl App {
    fn new() -> App {
        App {
            state: TableState::default(),
            items: vec![
                vec!["Row11".to_string(), "Row12".to_string(), "Row13".to_string()],
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
        }
    }
    pub fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }
}

async fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> io::Result<()> {
    let active_player_json_locations = deserializer::JSONDataLocations {
        url: env::var("ACTIVE_PLAYER_URL").unwrap(),
        json: env::var("ACTIVE_PLAYER_JSON").unwrap(),
    };

    let all_player_json_locations = deserializer::JSONDataLocations {
        url: env::var("ALL_PLAYERS_URL").unwrap(),
        json: env::var("ALL_PLAYERS_JSON").unwrap(),
    };

    let client: Client = network::build_client().await;

    let deserializer_params = deserializer::DeserializerParams {
        use_sample_json: true,
        active_player_json_locations,
        all_player_json_locations,
        client: &client,
    };

    if deserializer_params.use_sample_json {
        info!("use_sample_json is true. Using JSON files in resources dir.");
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

    loop {
        let (active_player_data, all_player_data) =
            deserializer::deserializer(&deserializer_params).await;

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

        // let display_data = ;

        app.items = display_data(
            &champion,
            active_player_data,
            ability_ranks,
            opponant_team,
            resistance,
        );
        // println!("================================");

        // Sleep for 5 seconds between running the loop again to save resources
        // sleep(Duration::from_secs(
        //     env::var("SAMPLE_RATE")
        //         .unwrap_or_else(|_| String::from("15"))
        //         .parse::<u64>()
        //         .unwrap_or(15),
        // ))
        // .await;
        terminal.draw(|f| ui(f, &mut app))?;

        // if let Event::Key(key) = event::read()? {
        //     match key.code {
        //         KeyCode::Char('q') => return Ok(()),
        //         _ => {}
        //     }
        // }

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

fn ui<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    let rects = Layout::default()
        .constraints([Constraint::Percentage(100)].as_ref())
        .split(f.size());

    let selected_style = Style::default().add_modifier(Modifier::REVERSED);
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
        .highlight_style(selected_style)
        .highlight_symbol(">> ")
        .widths(&[
            Constraint::Length(12),
            Constraint::Length(5),
            Constraint::Length(5),
        ]);
    f.render_stateful_widget(t, rects[0], &mut app.state);
    let tui_w: TuiLoggerWidget = TuiLoggerWidget::default()
        .block(
            Block::default()
                .title("Independent Tui Logger View")
                .border_style(Style::default().fg(Color::White).bg(Color::Black))
                .borders(Borders::ALL),
        )
        .output_separator('|')
        .output_timestamp(Some("%F %H:%M:%S%.3f".to_string()))
        .output_level(Some(TuiLoggerLevelOutput::Long))
        .output_target(false)
        .output_file(false)
        .output_line(false)
        .style(Style::default().fg(Color::White).bg(Color::Black));
    f.render_widget(tui_w, rects[1]);
}
