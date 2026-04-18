use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Sparkline},
};

use crate::collector::sensor::SensorCollector;

/// Render the sensor widget into the given area.
pub fn render(frame: &mut Frame, area: Rect, sensor: &SensorCollector) {
    let outer_block = Block::default()
        .title(" Sensors ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Magenta));

    let inner = outer_block.inner(area);
    frame.render_widget(outer_block, area);

    if inner.width < 4 || inner.height < 3 {
        return;
    }

    if !sensor.has_sensors() {
        let msg = Paragraph::new("No sensors detected")
            .style(Style::default().fg(Color::DarkGray))
            .alignment(ratatui::layout::Alignment::Center);
        frame.render_widget(msg, inner);
        return;
    }

    // Split vertically: top 40% for CPU temp sparkline, bottom 60% for sensor list.
    let vert = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(inner);

    render_sparkline(frame, vert[0], sensor);
    render_sensor_list(frame, vert[1], sensor);
}

/// Render the CPU temperature sparkline in the top section.
fn render_sparkline(frame: &mut Frame, area: Rect, sensor: &SensorCollector) {
    if area.height == 0 || area.width == 0 {
        return;
    }

    // Build label from current CPU temp.
    let label_text = match sensor.cpu_temp() {
        Some(t) => format!("CPU: {:.1}°C", t),
        None => "CPU: N/A".to_string(),
    };

    // Render label in top-right.
    let label_width = label_text.len() as u16;
    if area.width > label_width + 1 && area.height > 0 {
        let label_area = Rect::new(area.x + area.width - label_width, area.y, label_width, 1);
        let label = Paragraph::new(Span::styled(label_text, Style::default().fg(Color::White)));
        frame.render_widget(label, label_area);
    }

    // Sparkline for CPU temp history.
    let spark_area = if area.height > 1 {
        Rect::new(area.x, area.y + 1, area.width, area.height - 1)
    } else {
        area
    };

    // Determine the scale ceiling: use critical temp of the CPU sensor if known, otherwise 100°C.
    let max_temp = sensor
        .readings()
        .iter()
        .find(|r| Some(r.current_temp) == sensor.cpu_temp())
        .and_then(|r| r.critical_temp)
        .unwrap_or(100.0) as f64;

    let data: Vec<u64> = sensor
        .cpu_temp_history()
        .iter()
        .map(|&t| {
            let pct = if max_temp > 0.0 {
                (t / max_temp).clamp(0.0, 1.0)
            } else {
                0.0
            };
            (pct * 10_000.0) as u64
        })
        .collect();

    let sparkline = Sparkline::default()
        .data(&data)
        .max(10_000)
        .style(Style::default().fg(Color::Magenta));

    frame.render_widget(sparkline, spark_area);
}

/// Render the list of sensor readings in the bottom section.
fn render_sensor_list(frame: &mut Frame, area: Rect, sensor: &SensorCollector) {
    let max_lines = area.height as usize;
    if max_lines == 0 {
        return;
    }

    let mut sorted: Vec<_> = sensor.readings().to_vec();
    sorted.sort_by(|a, b| a.label.cmp(&b.label));

    let mut lines: Vec<Line> = Vec::with_capacity(max_lines);

    for reading in sorted.iter().take(max_lines) {
        let color = temp_color(reading.current_temp, reading.critical_temp);
        let temp_str = format!("{}: {:.1}°C", reading.label, reading.current_temp);

        let mut spans = vec![Span::styled(temp_str, Style::default().fg(color))];

        if let Some(crit) = reading.critical_temp {
            if reading.current_temp >= crit {
                spans.push(Span::styled(" (CRIT!)", Style::default().fg(Color::Red)));
            }
        }

        lines.push(Line::from(spans));
    }

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, area);
}

/// Choose a color based on temperature thresholds.
fn temp_color(temp: f32, critical: Option<f32>) -> Color {
    if let Some(crit) = critical {
        if temp >= crit {
            return Color::Red;
        }
    }
    if temp >= 80.0 {
        Color::Red
    } else if temp >= 60.0 {
        Color::Yellow
    } else {
        Color::Green
    }
}
