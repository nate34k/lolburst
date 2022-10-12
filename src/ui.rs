use std::{vec, io};

use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    symbols,
    text::Span,
    widgets::{Axis, Block, Borders, Cell, Chart, Dataset, GraphType, Paragraph, Row, Table, BorderType},
    Frame, Terminal, terminal::CompletedFrame,
};
use tui_logger::{TuiLoggerLevelOutput, TuiLoggerSmartWidget};

use crate::app::{self, Stats, App};

pub mod burst_table;
pub mod cs;
pub mod gold;
pub mod vs;

pub struct UI<'a, B> {
    pub main_block: Block<'a>,
    phantom: std::marker::PhantomData<B>,
}

impl<B: Backend> UI<'_, B> {
    pub fn new() -> Self {
        UI {
            main_block: Block::default().borders(Borders::ALL).border_type(BorderType::Double),
            phantom: std::marker::PhantomData,
        }
    }

    pub fn draw<'terminal>(
        &mut self,
        terminal: &'terminal mut Terminal<B>,
        app: &mut App,
    ) -> Result<CompletedFrame<'terminal>, std::io::Error> 
    where B: Backend
    {
        terminal.draw(|f| {
            let size = f.size();
            let block_inner = self.main_block.inner(size);
            f.render_widget(self.main_block.clone(), size);
            // Define a layout for main_block_inner
            // 
            // Either draw the app or both the app and the logger depending on
            // the app's logger_scroll_mode
            //
            // ---------------------------
            // |                         |
            // |           app           |
            // |                         |
            // |                         |
            // |                         |
            // ---------------------------
            //  or
            // --------------------------- 
            // |           app           |
            // |                         |
            // |-------------------------|
            // |         logger          |
            // |                         |
            // ---------------------------
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints(logger_constraint(app).as_ref(),
                )
                .split(block_inner);

            // Define a layout for the app area
            // 
            // ---------------------------
            // |                         |
            // |           app           |
            // |                         | -----
            // |                         |     |
            // |                         |     |
            // ---------------------------     |
            //                                 |
            // ---------------------------     |
            // |       |                 |     |
            // | Table |     Stats       |     |
            // |   35  |      100%       | <---|
            // |       |                 |
            // |       |                 |
            // ---------------------------
            let data_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(vec![Constraint::Length(35), Constraint::Percentage(100)])
                .split(chunks[0]);
            
            let stats_block = Block::default().borders(Borders::ALL).title("Stats");
            f.render_widget(stats_block.clone(), data_chunks[1]);

            // Define a layout for the stats area
            //
            // ---------------------------
            // |                         |
            // |          Stats          |
            // |                         | -----
            // |                         |     |
            // |                         |     |
            // ---------------------------     |
            //                                 |
            // ---------------------------     |
            // |       paragraph  3      |     |
            // |-------------------------|     |
            // |          charts         | <---|    
            // |           100%          |
            // |                         |
            // ---------------------------
            let stats_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints(vec![Constraint::Length(3), Constraint::Percentage(90)])
                .split(stats_block.inner(data_chunks[1]));

            // Split the paragraph into 3 chunks
            let paragraph_stats_rects = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(vec![
                    Constraint::Percentage(33),
                    Constraint::Percentage(34),
                    Constraint::Percentage(33),
                ])
                .split(stats_chunks[0]);

            // Split the charts into 3 chunks
            let chart_stats_rects = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(vec![
                    Constraint::Percentage(33),
                    Constraint::Percentage(34),
                    Constraint::Percentage(33),
                ])
                .split(stats_chunks[1]);

            self.draw_burst_table(app, data_chunks[0], f);
            self.draw_gold_per_min_paragraph(app, paragraph_stats_rects[0], f);
            
            self.draw_gold_per_min_chart(app, chart_stats_rects[0], f)
        })
    }

    fn draw_burst_table(&mut self, app: &mut App, chunk: Rect, f: &mut Frame<B>) {
        // Define formatting for burst table
        // Set the bg style
        let burst_normal_style = Style::default();
        // Set the header cell names and style
        let burst_header_cells = ["Champion", "Level", "Burst"]
            .iter()
            .map(|h| Cell::from(*h).style(Style::default().fg(Color::LightBlue)));
        // Set the header row
        let burst_header = Row::new(burst_header_cells)
            .style(burst_normal_style)
            .height(1)
            .bottom_margin(1);
        // Set table rows
        let burst_rows = app.burst_table_items.iter().enumerate().map(|item| {
            let height = item
                .1
                .iter()
                .map(|content| content.chars().filter(|c| *c == '\n').count())
                .max()
                .unwrap_or(0)
                + 1;
            let style: Style;
            if item.1.last().unwrap().parse::<f64>().unwrap()
                > app.burst_last[item.0].parse::<f64>().unwrap()
            {
                style = Style::default().fg(Color::LightGreen);
            } else if item.1.last().unwrap().parse::<f64>().unwrap()
                < app.burst_last[item.0].parse::<f64>().unwrap()
            {
                style = Style::default().fg(Color::Red);
            } else {
                style = Style::default();
            }
            let cells = item.1.iter().map(|c| Cell::from(c.as_str()).style(style));
            Row::new(cells).height(height as u16)
        });

        // Define the burst table
        let t = Table::new(burst_rows)
            .header(burst_header)
            .block(Block::default().borders(Borders::ALL).title("Burst"))
            .widths(&[
                Constraint::Length(12),
                Constraint::Length(5),
                Constraint::Length(5),
            ]);

        // Render the burst table
        f.render_widget(t, chunk);
    }

    fn draw_gold_per_min_paragraph(&mut self, app: &mut App, chunk: Rect, f: &mut Frame<B>) {
        // Define a layout for "gold per minute"
        // Set style to correct color for "gold per minute"
        let style: Style =
            match_paragraph_style("gold", app.gold.gold_per_min_vecdeque.back().unwrap().1);
        // Define paragraph for "gold per minute"
        let paragraph = Paragraph::new(app.gold.string_from_per_min())
            .style(style)
            .block(build_block("Gold Per Minute", style))
            .alignment(Alignment::Center);
        // Render paragraph for "gold per minute"
        f.render_widget(paragraph, chunk);
    }

    fn draw_gold_per_min_chart(&mut self, app: &mut App, chunk: Rect, f: &mut Frame<B>) {
        // Set style to correct color for "gold per minute"
        let style: Style =
            match_paragraph_style("gold", app.gold.gold_per_min_vecdeque.back().unwrap().1);

        // Build dataset for "gold per minute"
        let gold_per_min_dataset = vec![Dataset::default()
            .name("Gold Per Minute")
            .marker(symbols::Marker::Braille)
            .graph_type(GraphType::Line)
            .style(style)
            .data(&app.gold.gold_per_min_vecdeque)];

        // Build chart for "gold per minute"
        let c_gold = Chart::new(gold_per_min_dataset)
            .x_axis(
                Axis::default()
                    .title(Span::styled("T", Style::default().fg(Color::DarkGray)))
                    .style(Style::default())
                    .bounds(app.gold.x_axis_bounds)
                    .labels(
                        app.gold
                            .x_axis_labels
                            .iter()
                            .cloned()
                            .map(Span::from)
                            .collect(),
                    ),
            )
            .y_axis(
                Axis::default()
                    .title(Span::styled("GPM", Style::default().fg(Color::DarkGray)))
                    .style(Style::default())
                    .bounds(app.gold.y_axis_bounds)
                    .labels(
                        app.gold
                            .y_axis_labels
                            .iter()
                            .cloned()
                            .map(Span::from)
                            .collect(),
                    ),
            )
            .block(build_block("Gold Per Minute", Style::default()));
        
        // Render chart for "gold per minute"
        f.render_widget(c_gold, chunk);
    }
}

// TODO: refactor this mess
pub fn ui<B: Backend>(f: &mut Frame<B>, size: Rect, app: &app::App) {
    // Define a block ui element with a border and a title
    let main_block = Block::default().borders(Borders::ALL).title("lolburst");

    // Define an inner Rect for the block element
    let main_block_inner = main_block.inner(size);

    // Render the block element
    f.render_widget(main_block, size);

    let constraints = logger_constraint(app);
    
    let logger_style = logger_style(app);

    // Define a layout for main_block_inner
    // 
    // Either draw the app or both the app and the logger depending on
    // the app's logger_scroll_mode
    //
    // ---------------------------
    // |                         |
    // |           app           |
    // |                         |
    // |                         |
    // |                         |
    // ---------------------------
    //  or
    // --------------------------- 
    // |           app           |
    // |                         |
    // |-------------------------|
    // |         logger          |
    // |                         |
    // ---------------------------
    let ui_blocks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(main_block_inner);

    // Define a layout for the app area
    // 
    // ---------------------------
    // |                         |
    // |           app           |
    // |                         | -----
    // |                         |     |
    // |                         |     |
    // ---------------------------     |
    //                                 |
    // ---------------------------     |
    // |       |                 |     |
    // | Table |     Stats       |     |
    // |   35  |      100%       | <---|
    // |       |                 |
    // |       |                 |
    // ---------------------------
    let data_blocks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(vec![Constraint::Length(35), Constraint::Percentage(100)])
        .split(ui_blocks[0]);

    let stats_rects = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![Constraint::Length(3), Constraint::Percentage(90)])
        .split(data_blocks[1]);

    // Define a layout for stats rects
    let paragraph_stats_rects = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(vec![
            Constraint::Percentage(33),
            Constraint::Percentage(34),
            Constraint::Percentage(33),
        ])
        .split(stats_rects[0]);

    let chart_stats_rects = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(vec![
            Constraint::Percentage(33),
            Constraint::Percentage(34),
            Constraint::Percentage(33),
        ])
        .split(stats_rects[1]);

    // Define formatting for burst table
    // Set the bg style
    let burst_normal_style = Style::default();
    // Set the header cell names and style
    let burst_header_cells = ["Champion", "Level", "Burst"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(Color::LightBlue)));
    // Set the header row
    let burst_header = Row::new(burst_header_cells)
        .style(burst_normal_style)
        .height(1)
        .bottom_margin(1);
    // Set table rows
    let burst_rows = app.burst_table_items.iter().enumerate().map(|item| {
        let height = item
            .1
            .iter()
            .map(|content| content.chars().filter(|c| *c == '\n').count())
            .max()
            .unwrap_or(0)
            + 1;
        let style: Style;
        if item.1.last().unwrap().parse::<f64>().unwrap()
            > app.burst_last[item.0].parse::<f64>().unwrap()
        {
            style = Style::default().fg(Color::LightGreen);
        } else if item.1.last().unwrap().parse::<f64>().unwrap()
            < app.burst_last[item.0].parse::<f64>().unwrap()
        {
            style = Style::default().fg(Color::Red);
        } else {
            style = Style::default();
        }
        let cells = item.1.iter().map(|c| Cell::from(c.as_str()).style(style));
        Row::new(cells).height(height as u16)
    });

    // Define the burst table
    let t = Table::new(burst_rows)
        .header(burst_header)
        .block(Block::default().borders(Borders::ALL).title("burst"))
        .widths(&[
            Constraint::Length(12),
            Constraint::Length(5),
            Constraint::Length(5),
        ]);

    // Render the burst table
    f.render_widget(t, data_blocks[0]);

    // Helper closure for creating a Block for a paragraph
    let create_block = |title, style| {
        Block::default()
            .borders(Borders::ALL)
            .style(style)
            .title(Span::styled(
                title,
                Style::default().add_modifier(Modifier::BOLD),
            ))
    };

    // Define a layout for "gold per minute"
    // Set style to correct color for "gold per minute"
    let style: Style =
        match_paragraph_style("gold", app.gold.gold_per_min_vecdeque.back().unwrap().1);
    // Define paragraph for "gold per minute"
    let paragraph = Paragraph::new(app.gold.string_from_per_min())
        .style(style)
        .block(create_block("Gold Per Minute", style))
        .alignment(Alignment::Center);
    // Render paragraph for "gold per minute"
    f.render_widget(paragraph, paragraph_stats_rects[0]);
    // Build dataset for "gold per minute"
    let gold_per_min_dataset = vec![Dataset::default()
        .name("Gold Per Minute")
        .marker(symbols::Marker::Braille)
        .graph_type(GraphType::Line)
        .style(style)
        .data(&app.gold.gold_per_min_vecdeque)];

    // Build chart for "gold per minute"
    let c_gold = Chart::new(gold_per_min_dataset)
        .x_axis(
            Axis::default()
                .title(Span::styled("T", Style::default().fg(Color::DarkGray)))
                .style(Style::default())
                .bounds(app.gold.x_axis_bounds)
                .labels(
                    app.gold
                        .x_axis_labels
                        .iter()
                        .cloned()
                        .map(Span::from)
                        .collect(),
                ),
        )
        .y_axis(
            Axis::default()
                .title(Span::styled("GPM", Style::default().fg(Color::DarkGray)))
                .style(Style::default())
                .bounds(app.gold.y_axis_bounds)
                .labels(
                    app.gold
                        .y_axis_labels
                        .iter()
                        .cloned()
                        .map(Span::from)
                        .collect(),
                ),
        )
        .block(create_block("Gold Per Minute", Style::default()));

    // Render chart for "gold per minute"
    f.render_widget(c_gold, chart_stats_rects[0]);

    // Define a layout for "cs per minute"
    // Set style to correct color for "cs per minute"
    let style: Style = match_paragraph_style("cs", app.cs.cs_per_min_vecdeque.back().unwrap().1);

    // Define paragraph for "cs per minute"
    let paragraph = Paragraph::new(app.cs.string_from_per_min())
        .style(style)
        .block(create_block("CS Per Minute", style))
        .alignment(Alignment::Center);
    f.render_widget(paragraph, paragraph_stats_rects[1]);

    // Build dataset for "cs per minute"
    let cs_per_min_dataset = vec![Dataset::default()
        .name("CS Per Minute")
        .marker(symbols::Marker::Braille)
        .graph_type(GraphType::Line)
        .style(style)
        .data(&app.cs.cs_per_min_vecdeque)];

    // Build chart for "cs per minute"
    let c_cs = Chart::new(cs_per_min_dataset)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("CS Per Minute"),
        )
        .x_axis(
            Axis::default()
                .title(Span::styled("T", Style::default().fg(Color::DarkGray)))
                .style(Style::default())
                .bounds(app.cs.x_axis_bounds)
                .labels(
                    app.cs
                        .x_axis_labels
                        .iter()
                        .cloned()
                        .map(Span::from)
                        .collect(),
                ),
        )
        .y_axis(
            Axis::default()
                .title(Span::styled("CSPM", Style::default().fg(Color::DarkGray)))
                .style(Style::default())
                .bounds(app.cs.y_axis_bounds)
                .labels(
                    app.cs
                        .y_axis_labels
                        .iter()
                        .cloned()
                        .map(Span::from)
                        .collect(),
                ),
        );

    // Render chart for "cs per minute"
    f.render_widget(c_cs, chart_stats_rects[1]);

    // Define a layout for "vs per minute"
    // Set style to correct color for "vs per minute"
    let style: Style = match_paragraph_style("vs", app.vs.vs_per_min_vecdeque.back().unwrap().1);

    // Define paragraph for "vs per minute"
    let paragraph = Paragraph::new(app.vs.string_from_per_min())
        .style(style)
        .block(create_block("VS Per Minute", Style::default()))
        .alignment(Alignment::Center);
    f.render_widget(paragraph, paragraph_stats_rects[2]);

    // Build dataset for "vs per minute"
    let vs_per_min_dataset = vec![Dataset::default()
        .name("VS Per Minute")
        .marker(symbols::Marker::Braille)
        .graph_type(GraphType::Line)
        .style(style)
        .data(&app.vs.vs_per_min_vecdeque)];

    // Build chart for "vs per minute"
    let c_vs = Chart::new(vs_per_min_dataset)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("VS Per Minute"),
        )
        .x_axis(
            Axis::default()
                .title(Span::styled("T", Style::default().fg(Color::DarkGray)))
                .style(Style::default())
                .bounds(app.vs.x_axis_bounds)
                .labels(
                    app.vs
                        .x_axis_labels
                        .iter()
                        .cloned()
                        .map(Span::from)
                        .collect(),
                ),
        )
        .y_axis(
            Axis::default()
                .title(Span::styled("VSPM", Style::default().fg(Color::DarkGray)))
                .style(Style::default())
                .bounds(app.vs.y_axis_bounds)
                .labels(
                    app.vs
                        .y_axis_labels
                        .iter()
                        .cloned()
                        .map(Span::from)
                        .collect(),
                ),
        );

    // Render chart for "vs per minute"
    f.render_widget(c_vs, chart_stats_rects[2]);

    if app.draw_logger {
        let tui_sm = TuiLoggerSmartWidget::default()
            .style_error(Style::default().fg(Color::Red))
            .style_debug(Style::default().fg(Color::Green))
            .style_warn(Style::default().fg(Color::Yellow))
            .style_trace(Style::default().fg(Color::Magenta))
            .style_info(Style::default().fg(Color::Cyan))
            .output_separator('|')
            .output_timestamp(Some("%F %H:%M:%S%.3f".to_string()))
            .output_level(Some(TuiLoggerLevelOutput::Long))
            .output_target(true)
            .output_file(true)
            .output_line(true)
            .state(&app.logger_state)
            .highlight_style(Style::default().fg(Color::Red))
            .border_style(logger_style);
        f.render_widget(tui_sm, ui_blocks[1]);
    }
}

fn logger_constraint(app: &app::App) -> Vec<Constraint> {
    if app.draw_logger {
        return vec![Constraint::Length(16), Constraint::Percentage(100)];
    }
    vec![Constraint::Percentage(100)]
}

fn logger_style(app: &app::App) -> Style {
    if app.logger_scroll_freeze {
        return Style::default().fg(Color::Red);
    }
    Style::default()
}

// Function to create a block for paragraphs
fn build_block(title: &str, style: Style) -> Block {
    Block::default()
        .borders(Borders::ALL)
        .style(style)
        .title(Span::styled(
            title,
            Style::default().add_modifier(Modifier::BOLD),
        ))
}

// Function to match the stat and return the appropriate style
fn match_paragraph_style(stat: &str, n: f64) -> Style {
    let color = RColor::new();
    match stat {
        "gold" => match n as i64 {
            0..=199 => Style::default().fg(color.iron),
            200..=249 => Style::default().fg(color.bronze),
            250..=299 => Style::default().fg(color.silver),
            300..=349 => Style::default().fg(color.gold),
            350..=399 => Style::default().fg(color.platinum),
            400..=449 => Style::default().fg(color.diamond),
            450..=499 => Style::default().fg(color.master),
            500..=549 => Style::default().fg(color.grandmaster),
            550..=650 => Style::default()
                .fg(color.challenger)
                .add_modifier(Modifier::SLOW_BLINK),
            _ => Style::default(),
        },
        "cs" => match n as i64 {
            0..=3 => Style::default().fg(color.iron),
            4 => Style::default().fg(color.bronze),
            5 => Style::default().fg(color.silver),
            6 => Style::default().fg(color.gold),
            7 => Style::default().fg(color.platinum),
            8..=9 => Style::default().fg(color.diamond),
            10 => Style::default().fg(color.master),
            11 => Style::default().fg(color.grandmaster),
            12 => Style::default()
                .fg(color.challenger)
                .add_modifier(Modifier::SLOW_BLINK),
            _ => Style::default(),
        },
        "vs" => match n {
            n if n < 0.2 => Style::default().fg(color.iron),
            n if n < 0.4 => Style::default().fg(color.bronze),
            n if n < 0.6 => Style::default().fg(color.silver),
            n if n < 0.8 => Style::default().fg(color.gold),
            n if n < 1.0 => Style::default().fg(color.platinum),
            n if n < 1.2 => Style::default().fg(color.diamond),
            n if n < 1.4 => Style::default().fg(color.master),
            n if n < 1.6 => Style::default().fg(color.grandmaster),
            n if n < 4.0 => Style::default()
                .fg(color.challenger)
                .add_modifier(Modifier::SLOW_BLINK),
            _ => Style::default(),
        },
        _ => Style::default(),
    }
}

// Struct for holding default values for the color of the tiers
struct RColor {
    iron: tui::style::Color,
    bronze: tui::style::Color,
    silver: tui::style::Color,
    gold: tui::style::Color,
    platinum: tui::style::Color,
    diamond: tui::style::Color,
    master: tui::style::Color,
    grandmaster: tui::style::Color,
    challenger: tui::style::Color,
}

impl RColor {
    fn new() -> RColor {
        RColor {
            iron: Color::Rgb(81, 68, 68),
            bronze: Color::Rgb(127, 84, 20),
            silver: Color::Rgb(240, 240, 240),
            gold: Color::Rgb(228, 228, 126),
            platinum: Color::Rgb(123, 228, 172),
            diamond: Color::Rgb(81, 245, 250),
            master: Color::Rgb(159, 53, 220),
            grandmaster: Color::Rgb(255, 59, 20),
            challenger: Color::Rgb(102, 204, 255),
        }
    }
}
