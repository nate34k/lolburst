use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect, Alignment},
    style::{Color, Style, Modifier},
    widgets::{Block, Borders, Cell, Row, Table, Paragraph},
    Frame, text::Span,
};
use tui_logger::{TuiLoggerLevelOutput, TuiLoggerWidget};

use crate::app;

pub fn ui<B: Backend>(f: &mut Frame<B>, size: Rect, app: &mut app::App) {
    // Define a block ui element with a border and a title
    let block = Block::default().borders(Borders::ALL).title("lolburst");

    // Define an inner Rect for the block element
    let inner_area = block.inner(size);

    // Render the block element
    f.render_widget(block, size);

    // Define a layout for inner_area
    let rects = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![
            Constraint::Length(16),
            Constraint::Percentage(100),
        ])
        .split(inner_area);

    // Define a layout for data area
    let data_rects = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(vec![
            Constraint::Length(35),
            Constraint::Percentage(100),
        ])
        .split(rects[0]);

    // Define a layout for stats rects
    let stats_rects = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(vec![
            Constraint::Percentage(33),
            Constraint::Percentage(33),
            Constraint::Percentage(34),
        ])
        .split(data_rects[1]);

    // Define formatting for burst table
    // Set the bg style
    let burst_normal_style = Style::default().bg(Color::Blue);
    // Set the header cell names and style
    let burst_header_cells = ["Champion", "Level", "Burst"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(Color::DarkGray)));
    // Set the header row
    let burst_header = Row::new(burst_header_cells)
        .style(burst_normal_style)
        .height(1)
        .bottom_margin(1);
    // Set table rows
    let burst_rows = app.burst_table_items.iter().map(|item| {
        let height = item
            .iter()
            .map(|content| content.chars().filter(|c| *c == '\n').count())
            .max()
            .unwrap_or(0)
            + 1;
        let cells = item.iter().map(|c| Cell::from(c.as_str()));
        Row::new(cells).height(height as u16).bottom_margin(1)
    });

    // Define and render the burst table
    let t = Table::new(burst_rows)
        .header(burst_header)
        .block(Block::default().borders(Borders::ALL).title("burst"))
        .widths(&[
            Constraint::Length(12),
            Constraint::Length(5),
            Constraint::Length(5),
        ]);
    f.render_stateful_widget(t, data_rects[0], &mut app.burst_table_state);

    let create_block = |title| {
        Block::default()
            .borders(Borders::ALL)
            .style(Style::default().bg(Color::Black).fg(Color::White))
            .title(Span::styled(
                title,
                Style::default().add_modifier(Modifier::BOLD),
            ))
    };

    let paragraph = Paragraph::new(app.gold_per_min.clone())
        .style(Style::default().bg(Color::Black).fg(Color::White))
        .block(create_block("Gold Per Minute"))
        .alignment(Alignment::Center);
    f.render_widget(paragraph, stats_rects[0]);
    let paragraph = Paragraph::new(app.cs_per_min.clone())
        .style(Style::default().bg(Color::Black).fg(Color::White))
        .block(create_block("CS Per Minute"))
        .alignment(Alignment::Center);
    f.render_widget(paragraph, stats_rects[1]);
    let paragraph = Paragraph::new(app.vs_per_min.clone())
        .style(Style::default().bg(Color::Black).fg(Color::White))
        .block(create_block("VS Per Minute"))
        .alignment(Alignment::Center);
    f.render_widget(paragraph, stats_rects[2]);

    // Define formatting for log widget
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

    // Render the log widget
    f.render_widget(tui_w, rects[1]);
}
