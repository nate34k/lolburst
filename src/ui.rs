use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Cell, Row, Table},
    Frame,
};
use tui_logger::{TuiLoggerLevelOutput, TuiLoggerWidget};

use crate::app;

pub fn ui<B: Backend>(f: &mut Frame<B>, size: Rect, app: &mut app::App) {
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
