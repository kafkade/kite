use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Sparkline},
};

use crate::collector::network::NetworkCollector;
use crate::util::units::format_bytes;

/// Render the Network widget into the given area.
pub fn render(frame: &mut Frame, area: Rect, net: &NetworkCollector) {
    let outer_block = Block::default()
        .title(" Network ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));

    let inner = outer_block.inner(area);
    frame.render_widget(outer_block, area);

    if inner.width < 4 || inner.height < 3 {
        return;
    }

    let vert = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(inner);

    render_graph(frame, vert[0], net);
    render_details(frame, vert[1], net);
}

fn format_speed(bytes_sec: f64) -> String {
    format!("{}/s", format_bytes(bytes_sec as u64))
}

fn render_graph(frame: &mut Frame, area: Rect, net: &NetworkCollector) {
    let label = format!(
        "↓ {}  ↑ {}",
        format_speed(net.total_rx_bytes_sec()),
        format_speed(net.total_tx_bytes_sec()),
    );

    if area.height > 0 {
        let label_line = Paragraph::new(Line::from(Span::styled(
            label,
            Style::default().fg(Color::White),
        )));
        frame.render_widget(label_line, Rect::new(area.x, area.y, area.width, 1));
    }

    let spark_area = if area.height > 1 {
        Rect::new(area.x, area.y + 1, area.width, area.height - 1)
    } else {
        return;
    };

    let data: Vec<u64> = net.rx_history().iter().map(|v| *v as u64).collect();

    let sparkline = Sparkline::default()
        .data(&data)
        .style(Style::default().fg(Color::Blue));

    frame.render_widget(sparkline, spark_area);
}

fn render_details(frame: &mut Frame, area: Rect, net: &NetworkCollector) {
    let mut lines: Vec<Line> = Vec::new();
    let max_lines = area.height as usize;

    let interfaces = net.interfaces();
    let many = interfaces.len() > max_lines.saturating_sub(1);

    for iface in interfaces {
        if many && iface.rx_bytes_sec == 0.0 && iface.tx_bytes_sec == 0.0 {
            continue;
        }
        if lines.len() + 1 >= max_lines {
            break;
        }

        let name = format!("{:<12}", iface.name);
        let speeds = format!(
            "↓ {}  ↑ {}",
            format_speed(iface.rx_bytes_sec),
            format_speed(iface.tx_bytes_sec),
        );

        lines.push(Line::from(vec![
            Span::styled(name, Style::default().fg(Color::White)),
            Span::styled(speeds, Style::default().fg(Color::DarkGray)),
        ]));
    }

    // Totals line
    let totals = format!(
        "Total  ↓ {}  ↑ {}",
        format_bytes(net.total_rx_bytes()),
        format_bytes(net.total_tx_bytes()),
    );
    lines.push(Line::from(Span::styled(
        totals,
        Style::default().fg(Color::Green),
    )));

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, area);
}
