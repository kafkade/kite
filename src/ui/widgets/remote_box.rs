use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

use crate::collector::remote::{ConnectionState, RemoteCollector, RemoteSnapshot};
use crate::ui::theme::Theme;
use crate::util::units;

// Column widths
const COL_NAME: u16 = 16;
const COL_STATUS: u16 = 8;
const COL_CPU: u16 = 7;
const COL_MEM: u16 = 7;
const COL_DISK: u16 = 7;
const COL_NET: u16 = 16;
const COL_LATENCY: u16 = 8;

pub struct RemoteWidget {
    pub selected_index: usize,
    pub scroll_offset: usize,
}

#[allow(dead_code)]
impl RemoteWidget {
    pub fn new() -> Self {
        Self {
            selected_index: 0,
            scroll_offset: 0,
        }
    }

    pub fn scroll_up(&mut self) {
        self.selected_index = self.selected_index.saturating_sub(1);
    }

    pub fn scroll_down(&mut self, total: usize) {
        if total > 0 && self.selected_index < total - 1 {
            self.selected_index += 1;
        }
    }

    fn ensure_visible(&mut self, visible_height: usize) {
        if visible_height == 0 {
            return;
        }
        if self.selected_index < self.scroll_offset {
            self.scroll_offset = self.selected_index;
        } else if self.selected_index >= self.scroll_offset + visible_height {
            self.scroll_offset = self.selected_index - visible_height + 1;
        }
    }
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else if max > 1 {
        format!("{}…", &s[..max - 1])
    } else {
        s.chars().take(max).collect()
    }
}

fn state_indicator(state: &ConnectionState) -> &'static str {
    match state {
        ConnectionState::Connected => "●",
        ConnectionState::Connecting => "◌",
        ConnectionState::Disconnected => "○",
        ConnectionState::Error(_) => "✗",
    }
}

fn state_color(state: &ConnectionState, theme: &Theme) -> Color {
    match state {
        ConnectionState::Connected => theme.good,
        ConnectionState::Connecting => theme.warning,
        ConnectionState::Disconnected => theme.text_secondary,
        ConnectionState::Error(_) => theme.critical,
    }
}

fn build_header_line(width: u16, theme: &Theme) -> Line<'static> {
    let uptime_width = width.saturating_sub(
        COL_NAME + COL_STATUS + COL_CPU + COL_MEM + COL_DISK + COL_NET + COL_LATENCY,
    );
    let hdr_style = Style::default()
        .fg(theme.text_secondary)
        .add_modifier(Modifier::BOLD);

    let text = format!(
        "{:<name_w$}{:<st_w$}{:>cpu_w$}{:>mem_w$}{:>disk_w$}{:>net_w$}{:>lat_w$}{:>up_w$}",
        "Name",
        "Status",
        "CPU%",
        "Mem%",
        "Disk%",
        "Net ↓/↑",
        "Latency",
        "Uptime",
        name_w = COL_NAME as usize,
        st_w = COL_STATUS as usize,
        cpu_w = COL_CPU as usize,
        mem_w = COL_MEM as usize,
        disk_w = COL_DISK as usize,
        net_w = COL_NET as usize,
        lat_w = COL_LATENCY as usize,
        up_w = uptime_width as usize,
    );

    Line::from(Span::styled(text, hdr_style))
}

fn build_row_line(
    snap: &RemoteSnapshot,
    width: u16,
    selected: bool,
    theme: &Theme,
) -> Line<'static> {
    let uptime_width = width.saturating_sub(
        COL_NAME + COL_STATUS + COL_CPU + COL_MEM + COL_DISK + COL_NET + COL_LATENCY,
    );

    let base_style = if selected {
        Style::default()
            .bg(theme.selected_bg)
            .fg(theme.selected_fg)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(theme.text_primary)
    };

    let name_text = truncate(&snap.name, COL_NAME as usize - 1);
    let indicator = state_indicator(&snap.state);
    let st_color = if selected {
        theme.selected_fg
    } else {
        state_color(&snap.state, theme)
    };

    let is_connected = snap.state == ConnectionState::Connected;

    let cpu_text = if is_connected {
        format!("{:.1}%", snap.cpu_usage)
    } else {
        "N/A".to_string()
    };
    let cpu_color = if selected {
        theme.selected_fg
    } else if snap.cpu_usage > 90.0 {
        theme.critical
    } else if snap.cpu_usage > 70.0 {
        theme.warning
    } else {
        theme.good
    };

    let mem_text = if is_connected {
        format!("{:.1}%", snap.memory_percent())
    } else {
        "N/A".to_string()
    };
    let mem_color = if selected {
        theme.selected_fg
    } else if snap.memory_percent() > 90.0 {
        theme.critical
    } else if snap.memory_percent() > 70.0 {
        theme.warning
    } else {
        theme.good
    };

    let disk_text = if is_connected {
        format!("{:.1}%", snap.disk_percent())
    } else {
        "N/A".to_string()
    };

    let net_text = if is_connected {
        format!(
            "{}/{}",
            compact_bytes(snap.net_rx_rate),
            compact_bytes(snap.net_tx_rate)
        )
    } else {
        "N/A".to_string()
    };

    let lat_text = match snap.latency_ms {
        Some(ms) => format!("{}ms", ms),
        None => "N/A".to_string(),
    };

    let uptime_text = if is_connected && snap.uptime_secs > 0 {
        compact_duration(snap.uptime_secs)
    } else {
        "N/A".to_string()
    };

    let secondary_style = if selected {
        base_style
    } else {
        Style::default().fg(theme.text_secondary)
    };

    let spans = vec![
        Span::styled(
            format!("{:<w$}", name_text, w = COL_NAME as usize),
            base_style,
        ),
        Span::styled(
            format!(
                "{} {:<w$}",
                indicator,
                "",
                w = (COL_STATUS as usize).saturating_sub(2)
            ),
            base_style.fg(st_color),
        ),
        Span::styled(
            format!("{:>w$}", cpu_text, w = COL_CPU as usize),
            base_style.fg(if is_connected && !selected {
                cpu_color
            } else {
                base_style.fg.unwrap_or(theme.text_primary)
            }),
        ),
        Span::styled(
            format!("{:>w$}", mem_text, w = COL_MEM as usize),
            base_style.fg(if is_connected && !selected {
                mem_color
            } else {
                base_style.fg.unwrap_or(theme.text_primary)
            }),
        ),
        Span::styled(
            format!("{:>w$}", disk_text, w = COL_DISK as usize),
            secondary_style,
        ),
        Span::styled(
            format!("{:>w$}", net_text, w = COL_NET as usize),
            secondary_style,
        ),
        Span::styled(
            format!("{:>w$}", lat_text, w = COL_LATENCY as usize),
            secondary_style,
        ),
        Span::styled(
            format!("{:>w$}", uptime_text, w = uptime_width as usize),
            secondary_style,
        ),
    ];

    Line::from(spans)
}

/// Compact byte rate formatting (e.g., "1.5K", "3.2M").
fn compact_bytes(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{}B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.1}K", bytes as f64 / 1024.0)
    } else if bytes < 1024 * 1024 * 1024 {
        format!("{:.1}M", bytes as f64 / (1024.0 * 1024.0))
    } else {
        format!("{:.1}G", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
    }
}

/// Compact uptime formatting (e.g., "3d", "12h", "45m").
fn compact_duration(secs: u64) -> String {
    if secs < 60 {
        format!("{}s", secs)
    } else if secs < 3600 {
        format!("{}m", secs / 60)
    } else if secs < 86400 {
        format!("{}h", secs / 3600)
    } else {
        format!("{}d", secs / 86400)
    }
}

pub fn render(
    frame: &mut Frame,
    area: Rect,
    remote: &RemoteCollector,
    widget: &RemoteWidget,
    theme: &Theme,
) {
    let title = format!(
        " Remote ({}/{}) ",
        remote.connected_count(),
        remote.remote_count()
    );

    let outer_block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.remote_border));

    let inner = outer_block.inner(area);
    frame.render_widget(outer_block, area);

    if inner.width < 4 || inner.height < 3 {
        return;
    }

    // No remotes configured
    if !remote.has_remotes() {
        let msg = Paragraph::new(Line::from(Span::styled(
            if cfg!(feature = "ssh") {
                "No remotes configured — add [[remotes]] to config"
            } else {
                "SSH feature not enabled — build with --features ssh"
            },
            Style::default().fg(theme.text_secondary),
        )))
        .alignment(ratatui::layout::Alignment::Center);
        frame.render_widget(msg, inner);
        return;
    }

    let snapshots = remote.snapshots();

    // Layout: header(1) + list + footer(1)
    let show_footer = inner.height >= 5;
    let header_height = 1u16;
    let footer_height = if show_footer { 1u16 } else { 0u16 };
    let list_height = inner.height.saturating_sub(header_height + footer_height);

    let header_area = Rect::new(inner.x, inner.y, inner.width, header_height);
    let list_area = Rect::new(inner.x, inner.y + header_height, inner.width, list_height);
    let footer_area = if show_footer {
        Some(Rect::new(
            inner.x,
            inner.y + header_height + list_height,
            inner.width,
            footer_height,
        ))
    } else {
        None
    };

    // Header
    let header = build_header_line(inner.width, theme);
    frame.render_widget(Paragraph::new(header), header_area);

    // Remote rows
    let visible_height = list_height as usize;
    let mut widget_state = RemoteWidget {
        selected_index: widget.selected_index,
        scroll_offset: widget.scroll_offset,
    };
    widget_state.ensure_visible(visible_height);

    let end = (widget_state.scroll_offset + visible_height).min(snapshots.len());
    let start = widget_state.scroll_offset.min(end);

    let mut lines: Vec<Line> = Vec::new();
    for (i, snap) in snapshots.iter().enumerate().skip(start).take(end - start) {
        let selected = i == widget_state.selected_index;
        lines.push(build_row_line(snap, inner.width, selected, theme));
    }

    frame.render_widget(Paragraph::new(lines), list_area);

    // Footer with aggregate summary
    if let Some(fa) = footer_area {
        let mut parts = Vec::new();
        parts.push(format!(
            "Hosts: {}  Connected: {}",
            remote.remote_count(),
            remote.connected_count(),
        ));
        if let Some(avg_cpu) = remote.aggregate_cpu() {
            parts.push(format!("Avg CPU: {:.1}%", avg_cpu));
        }
        if let Some((used, total)) = remote.aggregate_memory() {
            parts.push(format!(
                "Total Mem: {}/{}",
                units::format_bytes(used),
                units::format_bytes(total)
            ));
        }

        let footer_line = Line::from(Span::styled(
            parts.join("  │  "),
            Style::default().fg(theme.text_secondary),
        ));
        frame.render_widget(Paragraph::new(footer_line), fa);
    }
}
