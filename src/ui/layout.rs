use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::Paragraph,
};

use crate::app::App;
use crate::ui::widgets::{cpu_box, mem_box};

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

    render_main_area(frame, outer[0], app);
    render_status_bar(frame, outer[1], app);
}

/// Render the main content area with CPU and Memory panels side by side.
fn render_main_area(frame: &mut Frame, area: Rect, app: &App) {
    let panels = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(50),
            Constraint::Percentage(50),
        ])
        .split(area);

    cpu_box::render(frame, panels[0], &app.cpu);
    mem_box::render(frame, panels[1], &app.mem);
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
