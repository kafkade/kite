use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Sparkline},
};

use crate::collector::cpu::CpuCollector;
use crate::ui::theme::Theme;
use crate::util::units::{format_duration, format_percentage};

/// Render the CPU widget into the given area.
pub fn render(frame: &mut Frame, area: Rect, cpu: &CpuCollector, theme: &Theme) {
    let outer_block = Block::default()
        .title(" CPU ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.cpu_border));

    let inner = outer_block.inner(area);
    frame.render_widget(outer_block, area);

    if inner.width < 4 || inner.height < 3 {
        return;
    }

    // Split vertically: top 60% graph, bottom 40% details
    let vert = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(inner);

    render_history(frame, vert[0], cpu, theme);
    render_bottom(frame, vert[1], cpu, theme);
}

fn render_history(frame: &mut Frame, area: Rect, cpu: &CpuCollector, theme: &Theme) {
    let usage_text = format_percentage(cpu.total_usage());

    // Usage label in top-right
    let label_width = usage_text.len() as u16;
    if area.width > label_width + 1 && area.height > 0 {
        let label_area = Rect::new(area.x + area.width - label_width, area.y, label_width, 1);
        let label = Paragraph::new(Span::styled(
            usage_text,
            Style::default().fg(theme.text_primary),
        ));
        frame.render_widget(label, label_area);
    }

    // Sparkline for history
    let spark_area = if area.height > 1 {
        Rect::new(area.x, area.y + 1, area.width, area.height - 1)
    } else {
        area
    };

    let data: Vec<u64> = cpu
        .usage_history()
        .iter()
        .map(|v| (v * 100.0) as u64)
        .collect();

    let sparkline = Sparkline::default()
        .data(&data)
        .max(10_000)
        .style(Style::default().fg(theme.sparkline_cpu));

    frame.render_widget(sparkline, spark_area);
}

fn render_bottom(frame: &mut Frame, area: Rect, cpu: &CpuCollector, theme: &Theme) {
    let horiz = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(area);

    render_core_bars(frame, horiz[0], cpu, theme);
    render_stats(frame, horiz[1], cpu, theme);
}

fn render_core_bars(frame: &mut Frame, area: Rect, cpu: &CpuCollector, theme: &Theme) {
    let cores = cpu.per_core_usage();
    let max_lines = area.height as usize;
    let show = cores.len().min(max_lines);

    let mut lines: Vec<Line> = Vec::with_capacity(show);
    for (i, &usage) in cores.iter().take(show).enumerate() {
        let label = format!("cpu{:<2}", i);
        let bar_width = area.width.saturating_sub(label.len() as u16 + 8) as usize;
        let filled = ((usage / 100.0) * bar_width as f64) as usize;
        let empty = bar_width.saturating_sub(filled);
        let pct_str = format!("{:>5.1}%", usage);

        let color = color_for_usage(usage, theme);
        lines.push(Line::from(vec![
            Span::styled(label, Style::default().fg(theme.text_secondary)),
            Span::raw(" "),
            Span::styled("█".repeat(filled), Style::default().fg(color)),
            Span::styled("░".repeat(empty), Style::default().fg(theme.text_secondary)),
            Span::raw(" "),
            Span::styled(pct_str, Style::default().fg(color)),
        ]));
    }

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, area);
}

fn color_for_usage(pct: f64, theme: &Theme) -> ratatui::style::Color {
    if pct >= 80.0 {
        theme.critical
    } else if pct >= 50.0 {
        theme.warning
    } else {
        theme.good
    }
}

fn render_stats(frame: &mut Frame, area: Rect, cpu: &CpuCollector, theme: &Theme) {
    let mut lines: Vec<Line> = Vec::new();

    // CPU model (truncate to fit)
    let max_w = area.width as usize;
    let model = cpu.cpu_model();
    let display_model = if model.len() > max_w {
        &model[..max_w]
    } else {
        model
    };
    lines.push(Line::from(Span::styled(
        display_model.to_string(),
        Style::default().fg(theme.text_primary),
    )));

    // Cores / threads
    lines.push(Line::from(Span::styled(
        format!("{}C/{}T", cpu.core_count(), cpu.thread_count()),
        Style::default().fg(theme.accent),
    )));

    // Average frequency
    let freqs = cpu.frequencies();
    if !freqs.is_empty() {
        let avg_mhz: f64 = freqs.iter().sum::<u64>() as f64 / freqs.len() as f64;
        let avg_ghz = avg_mhz / 1000.0;
        lines.push(Line::from(Span::styled(
            format!("{:.2} GHz", avg_ghz),
            Style::default().fg(theme.accent),
        )));
    }

    // Load averages (skip if all zeros, i.e. Windows)
    let (l1, l5, l15) = cpu.load_averages();
    if l1 != 0.0 || l5 != 0.0 || l15 != 0.0 {
        lines.push(Line::from(Span::styled(
            format!("Load: {:.2} {:.2} {:.2}", l1, l5, l15),
            Style::default().fg(theme.warning),
        )));
    }

    // Uptime
    lines.push(Line::from(Span::styled(
        format!("Up: {}", format_duration(cpu.uptime_secs())),
        Style::default().fg(theme.good),
    )));

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, area);
}
