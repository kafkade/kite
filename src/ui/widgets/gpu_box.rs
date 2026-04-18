use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Sparkline},
};

use crate::collector::gpu::GpuCollector;
use crate::util::units::{format_bytes, format_percentage};

/// Render the GPU widget into the given area.
pub fn render(frame: &mut Frame, area: Rect, gpu: &GpuCollector) {
    let outer_block = Block::default()
        .title(" GPU ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Blue));

    let inner = outer_block.inner(area);
    frame.render_widget(outer_block, area);

    if inner.width < 4 || inner.height < 3 {
        return;
    }

    if !gpu.has_gpu() {
        let msg = Paragraph::new("No GPU detected")
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center);
        let y_offset = inner.height / 2;
        let centered = Rect::new(inner.x, inner.y + y_offset, inner.width, 1);
        frame.render_widget(msg, centered);
        return;
    }

    let vert = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(inner);

    render_history(frame, vert[0], gpu);
    render_details(frame, vert[1], gpu);
}

fn render_history(frame: &mut Frame, area: Rect, gpu: &GpuCollector) {
    let current_pct = gpu
        .devices()
        .first()
        .and_then(|d| d.utilization_gpu)
        .map(|u| format_percentage(u as f64))
        .unwrap_or_else(|| "N/A".to_string());

    // Usage label in top-right
    let label_width = current_pct.len() as u16;
    if area.width > label_width + 1 && area.height > 0 {
        let label_area = Rect::new(area.x + area.width - label_width, area.y, label_width, 1);
        let label = Paragraph::new(Span::styled(current_pct, Style::default().fg(Color::White)));
        frame.render_widget(label, label_area);
    }

    let spark_area = if area.height > 1 {
        Rect::new(area.x, area.y + 1, area.width, area.height - 1)
    } else {
        area
    };

    let data: Vec<u64> = gpu
        .gpu_usage_history()
        .iter()
        .map(|v| (v * 100.0) as u64)
        .collect();

    let sparkline = Sparkline::default()
        .data(&data)
        .max(10_000)
        .style(Style::default().fg(Color::Magenta));

    frame.render_widget(sparkline, spark_area);
}

fn color_for_usage(pct: u32) -> Color {
    if pct >= 80 {
        Color::Red
    } else if pct >= 50 {
        Color::Yellow
    } else {
        Color::Green
    }
}

fn color_for_temp(temp: u32) -> Color {
    if temp >= 80 {
        Color::Red
    } else if temp >= 60 {
        Color::Yellow
    } else {
        Color::Green
    }
}

fn render_details(frame: &mut Frame, area: Rect, gpu: &GpuCollector) {
    let horiz = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(area);

    render_left(frame, horiz[0], gpu);
    render_right(frame, horiz[1], gpu);
}

fn render_left(frame: &mut Frame, area: Rect, gpu: &GpuCollector) {
    let dev = match gpu.devices().first() {
        Some(d) => d,
        None => return,
    };

    let mut lines: Vec<Line> = Vec::new();

    // GPU name (truncated to fit)
    let max_w = area.width as usize;
    let display_name = if dev.name.len() > max_w {
        &dev.name[..max_w]
    } else {
        &dev.name
    };
    lines.push(Line::from(Span::styled(
        display_name.to_string(),
        Style::default().fg(Color::White),
    )));

    // Multi-GPU note
    if gpu.device_count() > 1 {
        lines.push(Line::from(Span::styled(
            format!("(+{} more)", gpu.device_count() - 1),
            Style::default().fg(Color::DarkGray),
        )));
    }

    // GPU utilization bar
    if let Some(util) = dev.utilization_gpu {
        let label = "GPU ";
        let bar_width = area.width.saturating_sub(label.len() as u16 + 8) as usize;
        let filled = ((util as f64 / 100.0) * bar_width as f64) as usize;
        let empty = bar_width.saturating_sub(filled);
        let pct_str = format!("{:>5.1}%", util as f64);
        let color = color_for_usage(util);

        lines.push(Line::from(vec![
            Span::styled(label, Style::default().fg(Color::DarkGray)),
            Span::styled("█".repeat(filled), Style::default().fg(color)),
            Span::styled("░".repeat(empty), Style::default().fg(Color::DarkGray)),
            Span::raw(" "),
            Span::styled(pct_str, Style::default().fg(color)),
        ]));
    } else {
        lines.push(Line::from(Span::styled(
            "GPU  N/A",
            Style::default().fg(Color::DarkGray),
        )));
    }

    // VRAM bar
    match (dev.vram_used, dev.vram_total) {
        (Some(used), Some(total)) if total > 0 => {
            let pct = (used as f64 / total as f64 * 100.0) as u32;
            let label = "VRAM";
            let vram_text = format!(" {}/{}", format_bytes(used), format_bytes(total));
            let bar_width = area
                .width
                .saturating_sub(label.len() as u16 + vram_text.len() as u16 + 1)
                as usize;
            let filled = ((used as f64 / total as f64) * bar_width as f64) as usize;
            let empty = bar_width.saturating_sub(filled);
            let color = color_for_usage(pct);

            lines.push(Line::from(vec![
                Span::styled(label, Style::default().fg(Color::DarkGray)),
                Span::styled("█".repeat(filled), Style::default().fg(color)),
                Span::styled("░".repeat(empty), Style::default().fg(Color::DarkGray)),
                Span::styled(vram_text, Style::default().fg(color)),
            ]));
        }
        _ => {
            lines.push(Line::from(Span::styled(
                "VRAM N/A",
                Style::default().fg(Color::DarkGray),
            )));
        }
    }

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, area);
}

fn render_right(frame: &mut Frame, area: Rect, gpu: &GpuCollector) {
    let dev = match gpu.devices().first() {
        Some(d) => d,
        None => return,
    };

    let na = Span::styled("N/A", Style::default().fg(Color::DarkGray));
    let mut lines: Vec<Line> = Vec::new();

    // Temperature
    let temp_span = match dev.temperature {
        Some(t) => Span::styled(format!("{}°C", t), Style::default().fg(color_for_temp(t))),
        None => na.clone(),
    };
    lines.push(Line::from(vec![
        Span::styled("Temp: ", Style::default().fg(Color::DarkGray)),
        temp_span,
    ]));

    // Fan speed
    let fan_span = match dev.fan_speed {
        Some(f) => Span::styled(format!("{}%", f), Style::default().fg(Color::Cyan)),
        None => na.clone(),
    };
    lines.push(Line::from(vec![
        Span::styled("Fan:  ", Style::default().fg(Color::DarkGray)),
        fan_span,
    ]));

    // Graphics clock
    let gclk_span = match dev.clock_graphics {
        Some(c) => Span::styled(format!("{} MHz", c), Style::default().fg(Color::Cyan)),
        None => na.clone(),
    };
    lines.push(Line::from(vec![
        Span::styled("GClk: ", Style::default().fg(Color::DarkGray)),
        gclk_span,
    ]));

    // Memory clock
    let mclk_span = match dev.clock_memory {
        Some(c) => Span::styled(format!("{} MHz", c), Style::default().fg(Color::Cyan)),
        None => na.clone(),
    };
    lines.push(Line::from(vec![
        Span::styled("MClk: ", Style::default().fg(Color::DarkGray)),
        mclk_span,
    ]));

    // Power
    let power_span = match (dev.power_draw, dev.power_limit) {
        (Some(draw), Some(limit)) => Span::styled(
            format!("{:.0}W/{:.0}W", draw, limit),
            Style::default().fg(Color::Cyan),
        ),
        (Some(draw), None) => {
            Span::styled(format!("{:.0}W", draw), Style::default().fg(Color::Cyan))
        }
        _ => na.clone(),
    };
    lines.push(Line::from(vec![
        Span::styled("Pwr:  ", Style::default().fg(Color::DarkGray)),
        power_span,
    ]));

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, area);
}
