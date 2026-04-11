use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

use crate::app::App;

/// Render the full application layout.
pub fn render(frame: &mut Frame, app: &App) {
    let size = frame.area();

    // Split into: main content area + status bar (1 line)
    let outer = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(3),    // main content
            Constraint::Length(1), // status bar
        ])
        .split(size);

    render_main_area(frame, outer[0]);
    render_status_bar(frame, outer[1], app);
}

/// Render the main content area (placeholder for Stage 3+).
fn render_main_area(frame: &mut Frame, area: Rect) {
    let block = Block::default()
        .title(" Kite ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let welcome = Paragraph::new(vec![
        Line::from(""),
        Line::from(Span::styled(
            "  Welcome to Kite — system resource monitor",
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "  Press 'q' to quit  |  '?' for help",
            Style::default().fg(Color::DarkGray),
        )),
    ])
    .block(block);

    frame.render_widget(welcome, area);
}

/// Render the bottom status bar.
fn render_status_bar(frame: &mut Frame, area: Rect, app: &App) {
    let now = chrono::Local::now().format("%H:%M:%S").to_string();
    let uptime_str = crate::util::units::format_duration(app.uptime_secs());

    let left = format!(
        " Kite v{} │ {} ",
        env!("CARGO_PKG_VERSION"),
        app.hostname()
    );
    let right = format!(
        " {} │ up {} │ ↻ {}ms ",
        now,
        uptime_str,
        app.update_interval_ms()
    );

    let padding = area
        .width
        .saturating_sub((left.len() + right.len()) as u16);

    let bar = Line::from(vec![
        Span::styled(&left, Style::default().fg(Color::Cyan)),
        Span::raw(" ".repeat(padding as usize)),
        Span::styled(&right, Style::default().fg(Color::DarkGray)),
    ]);

    let bar_widget = Paragraph::new(bar)
        .style(Style::default().bg(Color::DarkGray).fg(Color::White));

    frame.render_widget(bar_widget, area);
}
