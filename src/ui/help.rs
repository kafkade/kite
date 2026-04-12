use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Flex, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
};

/// Render the help overlay centered on screen.
pub fn render(frame: &mut Frame) {
    let lines = build_help_lines();
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
        .border_style(Style::default().fg(Color::Cyan));

    let paragraph = Paragraph::new(lines).block(block);
    frame.render_widget(paragraph, area);
}

fn build_help_lines<'a>() -> Vec<Line<'a>> {
    vec![
        // Blank line after top border
        Line::from(""),
        // General section
        section_header("General"),
        key_line("  q / Ctrl+C", "Quit"),
        key_line("  ?", "Toggle help"),
        key_line("  m", "Settings menu"),
        key_line("  r", "Force refresh"),
        key_line("  Esc", "Close overlay/clear"),
        Line::from(""),
        // Process List section
        section_header("Process List"),
        key_line("  ↑/↓ or j/k", "Scroll"),
        key_line("  PgUp/PgDn", "Page scroll"),
        key_line("  ←/→", "Change sort column"),
        key_line("  Space", "Pause/unpause"),
        key_line("  /", "Filter processes"),
        key_line("  t", "Toggle tree view"),
        key_line("  K", "Kill process"),
        Line::from(""),
        // Navigation section
        section_header("Navigation"),
        key_line("  Tab", "Next panel"),
        key_line("  Shift+Tab", "Previous panel"),
        Line::from(""),
        // Footer hint
        Line::from(Span::styled(
            "Press ? to close",
            Style::default().fg(Color::DarkGray),
        ))
        .alignment(Alignment::Center),
    ]
}

fn section_header<'a>(title: &'a str) -> Line<'a> {
    Line::from(Span::styled(
        format!("  {title}"),
        Style::default()
            .fg(Color::White)
            .add_modifier(Modifier::BOLD),
    ))
}

fn key_line<'a>(key: &'a str, desc: &'a str) -> Line<'a> {
    Line::from(vec![
        Span::styled(format!("{key:<16}"), Style::default().fg(Color::Yellow)),
        Span::styled(desc, Style::default().fg(Color::White)),
    ])
}
