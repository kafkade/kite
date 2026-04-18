use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Flex, Layout},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
};

use crate::ui::theme::Theme;

/// Render the help overlay centered on screen.
pub fn render(frame: &mut Frame, theme: &Theme) {
    let lines = build_help_lines(theme);
    let content_height = lines.len() as u16 + 2; // +2 for border
    let width: u16 = 50;

    // Clear the entire terminal area behind the overlay.
    frame.render_widget(Clear, frame.area());

    // Center the box.
    let [area] = Layout::vertical([Constraint::Length(content_height)])
        .flex(Flex::Center)
        .areas(frame.area());
    let [area] = Layout::horizontal([Constraint::Length(width)])
        .flex(Flex::Center)
        .areas(area);

    let block = Block::default()
        .title(" Kite — Help ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.dialog_border));

    let paragraph = Paragraph::new(lines).block(block);
    frame.render_widget(paragraph, area);
}

fn build_help_lines<'a>(theme: &Theme) -> Vec<Line<'a>> {
    vec![
        // Blank line after top border
        Line::from(""),
        // General section
        section_header("General", theme),
        key_line("  q / Ctrl+C", "Quit", theme),
        key_line("  ?", "Toggle help", theme),
        key_line("  m", "Settings menu", theme),
        key_line("  r", "Force refresh", theme),
        key_line("  Esc", "Close overlay/clear", theme),
        Line::from(""),
        // Process List section
        section_header("Process List", theme),
        key_line("  ↑/↓ or j/k", "Scroll", theme),
        key_line("  PgUp/PgDn", "Page scroll", theme),
        key_line("  ←/→", "Change sort column", theme),
        key_line("  Space", "Pause/unpause", theme),
        key_line("  /", "Filter processes", theme),
        key_line("  t", "Toggle tree view", theme),
        key_line("  K", "Kill process", theme),
        Line::from(""),
        // Navigation section
        section_header("Navigation", theme),
        key_line("  Tab", "Next panel", theme),
        key_line("  Shift+Tab", "Previous panel", theme),
        Line::from(""),
        // Remote section
        section_header("Remote Monitoring", theme),
        key_line("  [[remotes]]", "Configure in config.toml", theme),
        Line::from(""),
        // Footer hint
        Line::from(Span::styled(
            "Press ? to close",
            Style::default().fg(theme.text_secondary),
        ))
        .alignment(Alignment::Center),
    ]
}

fn section_header<'a>(title: &'a str, theme: &Theme) -> Line<'a> {
    Line::from(Span::styled(
        format!("  {title}"),
        Style::default()
            .fg(theme.text_primary)
            .add_modifier(Modifier::BOLD),
    ))
}

fn key_line<'a>(key: &'a str, desc: &'a str, theme: &Theme) -> Line<'a> {
    Line::from(vec![
        Span::styled(format!("{key:<16}"), Style::default().fg(theme.warning)),
        Span::styled(desc, Style::default().fg(theme.text_primary)),
    ])
}
