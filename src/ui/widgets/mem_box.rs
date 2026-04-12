use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Sparkline},
};

use crate::collector::memory::MemoryCollector;
use crate::util::units::{format_bytes, format_percentage};

/// Build a text-based gauge bar like `"RAM  [████████░░░░░░░░░░░░░] 39.1%"`
fn render_gauge(label: &str, percent: f64, bar_width: usize, color: Color) -> Line<'static> {
    let filled = ((percent / 100.0) * bar_width as f64).round() as usize;
    let filled = filled.min(bar_width);
    let empty = bar_width.saturating_sub(filled);

    let bar_filled: String = "█".repeat(filled);
    let bar_empty: String = "░".repeat(empty);

    Line::from(vec![
        Span::styled(format!("{:<4} [", label), Style::default().fg(Color::White)),
        Span::styled(bar_filled, Style::default().fg(color)),
        Span::styled(bar_empty, Style::default().fg(Color::DarkGray)),
        Span::styled(
            format!("] {}", format_percentage(percent)),
            Style::default().fg(Color::White),
        ),
    ])
}

/// Pick a color based on usage percentage thresholds.
fn usage_color(percent: f64) -> Color {
    if percent >= 80.0 {
        Color::Red
    } else if percent >= 50.0 {
        Color::Yellow
    } else {
        Color::Green
    }
}

pub fn render(frame: &mut Frame, area: Rect, mem: &MemoryCollector) {
    let block = Block::default()
        .title(" Memory ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Green));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let sections = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(40),
            Constraint::Percentage(30),
            Constraint::Percentage(30),
        ])
        .split(inner);

    // --- Top section: RAM history sparkline ---
    let history_data: Vec<u64> = mem
        .ram_history()
        .iter()
        .map(|v| (v * 100.0) as u64)
        .collect();

    let sparkline = Sparkline::default()
        .data(&history_data)
        .max(100)
        .style(Style::default().fg(Color::Green));
    frame.render_widget(sparkline, sections[0]);

    // --- Middle section: Gauges ---
    let ram_pct = mem.ram_usage_percent();
    let bar_width = (sections[1].width as usize).saturating_sub(14); // account for label + brackets + pct

    let mut gauge_lines = vec![render_gauge(
        "RAM",
        ram_pct,
        bar_width,
        usage_color(ram_pct),
    )];

    if mem.swap_total() > 0 {
        let swap_pct = mem.swap_usage_percent();
        gauge_lines.push(render_gauge("SWP", swap_pct, bar_width, Color::Cyan));
    }

    // Add label line showing used/total
    let ram_label = format!(
        "RAM: {}/{} ({})",
        format_bytes(mem.used_ram()),
        format_bytes(mem.total_ram()),
        format_percentage(ram_pct),
    );
    gauge_lines.insert(
        0,
        Line::from(Span::styled(
            ram_label,
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )),
    );

    let gauges = Paragraph::new(gauge_lines);
    frame.render_widget(gauges, sections[1]);

    // --- Bottom section: Text stats ---
    let mut stat_lines = vec![
        Line::from(format!(
            "Total: {}  Used: {}  Free: {}",
            format_bytes(mem.total_ram()),
            format_bytes(mem.used_ram()),
            format_bytes(mem.free_ram()),
        )),
        Line::from(format!("Available: {}", format_bytes(mem.available_ram()))),
    ];

    if mem.swap_total() > 0 {
        stat_lines.push(Line::from(format!(
            "Swap: {}/{}",
            format_bytes(mem.swap_used()),
            format_bytes(mem.swap_total()),
        )));
    }

    let stats = Paragraph::new(stat_lines).style(Style::default().fg(Color::White));
    frame.render_widget(stats, sections[2]);
}
