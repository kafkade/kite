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
    let mut lines: Vec<Line> = Vec::new();

    // Blank line after top border
    lines.push(Line::from(""));

    // General section
    lines.push(section_header("General"));
    lines.push(key_line("  q / Ctrl+C", "Quit"));
    lines.push(key_line("  ?", "Toggle help"));
    lines.push(key_line("  m", "Settings menu"));
    lines.push(key_line("  r", "Force refresh"));
    lines.push(key_line("  Esc", "Close overlay/clear"));
    lines.push(Line::from(""));

    // Process List section
    lines.push(section_header("Process List"));
    lines.push(key_line("  ↑/↓ or j/k", "Scroll"));
    lines.push(key_line("  PgUp/PgDn", "Page scroll"));
    lines.push(key_line("  ←/→", "Change sort column"));
    lines.push(key_line("  Space", "Pause/unpause"));
    lines.push(key_line("  /", "Filter processes"));
    lines.push(key_line("  t", "Toggle tree view"));
    lines.push(key_line("  K", "Kill process"));
    lines.push(Line::from(""));

    // Navigation section
    lines.push(section_header("Navigation"));
    lines.push(key_line("  Tab", "Next panel"));
    lines.push(key_line("  Shift+Tab", "Previous panel"));
    lines.push(Line::from(""));

    // Footer hint
    lines.push(Line::from(Span::styled(
        "Press ? to close",
        Style::default().fg(Color::DarkGray),
    )).alignment(Alignment::Center));

    lines
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
        Span::styled(
            format!("{key:<16}"),
            Style::default().fg(Color::Yellow),
        ),
        Span::styled(desc, Style::default().fg(Color::White)),
    ])
}
