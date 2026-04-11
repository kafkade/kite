use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Sparkline},
    Frame,
};

use crate::collector::disk::DiskCollector;
use crate::util::units::{format_bytes, format_percentage};

/// Render the Disk widget into the given area.
pub fn render(frame: &mut Frame, area: Rect, disk: &DiskCollector) {
    let outer_block = Block::default()
        .title(" Disk ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Magenta));

    let inner = outer_block.inner(area);
    frame.render_widget(outer_block, area);

    if inner.width < 4 || inner.height < 3 {
        return;
    }

    let vert = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(35), Constraint::Percentage(65)])
        .split(inner);

    render_io_sparkline(frame, vert[0], disk);
    render_partition_list(frame, vert[1], disk);
}

fn render_io_sparkline(frame: &mut Frame, area: Rect, disk: &DiskCollector) {
    if area.height < 1 {
        return;
    }

    // I/O speed label
    let read_speed = format_bytes(disk.total_read_bytes_sec() as u64);
    let write_speed = format_bytes(disk.total_write_bytes_sec() as u64);
    let label = format!("R: {}/s  W: {}/s", read_speed, write_speed);

    let label_line = Line::from(Span::styled(label, Style::default().fg(Color::White)));
    let label_area = Rect::new(area.x, area.y, area.width, 1);
    frame.render_widget(Paragraph::new(label_line), label_area);

    // Sparkline below the label
    if area.height > 1 {
        let spark_area = Rect::new(area.x, area.y + 1, area.width, area.height - 1);

        let data: Vec<u64> = disk
            .read_history()
            .iter()
            .map(|v| *v as u64)
            .collect();

        if !data.is_empty() {
            let max_val = data.iter().copied().max().unwrap_or(1).max(1);
            let sparkline = Sparkline::default()
                .data(&data)
                .max(max_val)
                .style(Style::default().fg(Color::Magenta));
            frame.render_widget(sparkline, spark_area);
        }
    }
}

fn color_for_usage(pct: f64) -> Color {
    if pct >= 80.0 {
        Color::Red
    } else if pct >= 50.0 {
        Color::Yellow
    } else {
        Color::Green
    }
}

fn render_partition_list(frame: &mut Frame, area: Rect, disk: &DiskCollector) {
    let max_lines = area.height as usize;
    let disks = disk.disks();
    let show = disks.len().min(max_lines);

    let mut lines: Vec<Line> = Vec::with_capacity(show);

    for d in disks.iter().take(show) {
        let mount = &d.mount_point;
        let fs = &d.fs_type;
        let pct = d.usage_percent;
        let pct_str = format_percentage(pct);
        let used_str = format_bytes(d.used_bytes);
        let total_str = format_bytes(d.total_bytes);

        // Prefix: "C:\ [NTFS] "
        let prefix = format!("{} [{}] ", mount, fs);
        // Suffix: " 67.2%  120/256 GiB"
        let suffix = format!(" {}  {}/{}", pct_str, used_str, total_str);

        let bar_width = (area.width as usize)
            .saturating_sub(prefix.len() + suffix.len());
        let filled = ((pct / 100.0) * bar_width as f64) as usize;
        let empty = bar_width.saturating_sub(filled);

        let color = color_for_usage(pct);

        lines.push(Line::from(vec![
            Span::styled(prefix, Style::default().fg(Color::DarkGray)),
            Span::styled("█".repeat(filled), Style::default().fg(color)),
            Span::styled("░".repeat(empty), Style::default().fg(Color::DarkGray)),
            Span::styled(suffix, Style::default().fg(color)),
        ]));
    }

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, area);
}
