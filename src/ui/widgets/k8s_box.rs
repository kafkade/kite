use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

use crate::collector::k8s::{K8sCollector, PodInfo, PodStatus};
use crate::ui::theme::Theme;

// Column widths
const COL_NAME: u16 = 24;
const COL_NS: u16 = 14;
const COL_STATUS: u16 = 11;
const COL_READY: u16 = 6;
const COL_RESTARTS: u16 = 4;
const COL_AGE: u16 = 6;

pub struct K8sWidget {
    pub selected_index: usize,
    pub scroll_offset: usize,
}

#[allow(dead_code)]
impl K8sWidget {
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

fn format_age(duration: std::time::Duration) -> String {
    let secs = duration.as_secs();
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

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else if max > 1 {
        format!("{}…", &s[..max - 1])
    } else {
        s.chars().take(max).collect()
    }
}

fn status_color(status: PodStatus, theme: &Theme) -> Color {
    match status {
        PodStatus::Running | PodStatus::Succeeded => theme.good,
        PodStatus::Pending | PodStatus::ContainerCreating => theme.warning,
        PodStatus::Failed | PodStatus::CrashLoopBackOff | PodStatus::ImagePullBackOff => {
            theme.critical
        }
        PodStatus::Unknown | PodStatus::Terminating => theme.text_secondary,
    }
}

fn restart_color(restarts: u32, theme: &Theme) -> Color {
    if restarts > 5 {
        theme.critical
    } else if restarts > 0 {
        theme.warning
    } else {
        theme.text_primary
    }
}

fn ready_color(ready_str: &str, theme: &Theme) -> Color {
    // Parse "n/m" — red if n < m
    let parts: Vec<&str> = ready_str.split('/').collect();
    if parts.len() == 2 {
        let ready: u32 = parts[0].parse().unwrap_or(0);
        let total: u32 = parts[1].parse().unwrap_or(0);
        if total > 0 && ready < total {
            return theme.critical;
        }
    }
    theme.text_primary
}

fn build_header_line(width: u16, theme: &Theme) -> Line<'static> {
    let node_width =
        width.saturating_sub(COL_NAME + COL_NS + COL_STATUS + COL_READY + COL_RESTARTS + COL_AGE);
    let hdr_style = Style::default()
        .fg(theme.text_secondary)
        .add_modifier(Modifier::BOLD);

    let text = format!(
        "{:<name_w$}{:<ns_w$}{:<st_w$}{:>rdy_w$}{:>rst_w$}{:>age_w$}{:<node_w$}",
        "Name",
        "Namespace",
        "Status",
        "Ready",
        "↻",
        "Age",
        " Node",
        name_w = COL_NAME as usize,
        ns_w = COL_NS as usize,
        st_w = COL_STATUS as usize,
        rdy_w = COL_READY as usize,
        rst_w = COL_RESTARTS as usize,
        age_w = COL_AGE as usize,
        node_w = node_width as usize,
    );

    Line::from(Span::styled(text, hdr_style))
}

fn build_row_line(pod: &PodInfo, width: u16, selected: bool, theme: &Theme) -> Line<'static> {
    let node_width =
        width.saturating_sub(COL_NAME + COL_NS + COL_STATUS + COL_READY + COL_RESTARTS + COL_AGE);

    let base_style = if selected {
        Style::default()
            .bg(theme.selected_bg)
            .fg(theme.selected_fg)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(theme.text_primary)
    };

    let name_text = truncate(&pod.name, COL_NAME as usize);
    let ns_text = truncate(&pod.namespace, COL_NS as usize);
    let status_text = truncate(&pod.status.to_string(), COL_STATUS as usize);
    let ready_text = &pod.ready;
    let restart_text = pod.restarts.to_string();
    let age_text = format_age(pod.age);
    let node_text = truncate(&pod.node, node_width.saturating_sub(1) as usize);

    let st_color = if selected {
        theme.selected_fg
    } else {
        status_color(pod.status, theme)
    };
    let rdy_color = if selected {
        theme.selected_fg
    } else {
        ready_color(&pod.ready, theme)
    };
    let rst_color = if selected {
        theme.selected_fg
    } else {
        restart_color(pod.restarts, theme)
    };
    let ns_style = if selected {
        base_style
    } else {
        Style::default().fg(theme.text_secondary)
    };
    let node_style = if selected {
        base_style
    } else {
        Style::default().fg(theme.text_secondary)
    };

    let spans = vec![
        Span::styled(
            format!("{:<w$}", name_text, w = COL_NAME as usize),
            base_style,
        ),
        Span::styled(format!("{:<w$}", ns_text, w = COL_NS as usize), ns_style),
        Span::styled(
            format!("{:<w$}", status_text, w = COL_STATUS as usize),
            base_style.fg(st_color),
        ),
        Span::styled(
            format!("{:>w$}", ready_text, w = COL_READY as usize),
            base_style.fg(rdy_color),
        ),
        Span::styled(
            format!("{:>w$}", restart_text, w = COL_RESTARTS as usize),
            base_style.fg(rst_color),
        ),
        Span::styled(
            format!("{:>w$}", age_text, w = COL_AGE as usize),
            base_style,
        ),
        Span::styled(
            format!(
                " {:<w$}",
                node_text,
                w = node_width.saturating_sub(1) as usize
            ),
            node_style,
        ),
    ];

    Line::from(spans)
}

#[allow(dead_code)]
pub fn render(
    frame: &mut Frame,
    area: Rect,
    k8s: &K8sCollector,
    widget: &K8sWidget,
    theme: &Theme,
) {
    let ns_label = match k8s.namespace_filter() {
        Some(ns) => format!("ns: {}", ns),
        None => "all".to_string(),
    };
    let title = format!(" Kubernetes ({}) ", ns_label);

    let outer_block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.k8s_border));

    let inner = outer_block.inner(area);
    frame.render_widget(outer_block, area);

    if inner.width < 4 || inner.height < 3 {
        return;
    }

    // Kubernetes not available
    if !k8s.has_k8s() {
        let msg = Paragraph::new(Line::from(Span::styled(
            "Kubernetes not available",
            Style::default().fg(theme.text_secondary),
        )))
        .alignment(ratatui::layout::Alignment::Center);
        frame.render_widget(msg, inner);
        return;
    }

    let pods = k8s.pods();

    // No pods
    if pods.is_empty() {
        let msg = Paragraph::new(Line::from(Span::styled(
            "No pods",
            Style::default().fg(theme.text_secondary),
        )))
        .alignment(ratatui::layout::Alignment::Center);
        frame.render_widget(msg, inner);
        return;
    }

    // Layout: header(1) + pod list + footer(1)
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

    // Pod rows
    let visible_height = list_height as usize;
    let mut widget_state = K8sWidget {
        selected_index: widget.selected_index,
        scroll_offset: widget.scroll_offset,
    };
    widget_state.ensure_visible(visible_height);

    let end = (widget_state.scroll_offset + visible_height).min(pods.len());
    let start = widget_state.scroll_offset.min(end);

    let mut lines: Vec<Line> = Vec::new();
    for (i, pod) in pods.iter().enumerate().skip(start).take(end - start) {
        let selected = i == widget_state.selected_index;
        lines.push(build_row_line(pod, inner.width, selected, theme));
    }

    frame.render_widget(Paragraph::new(lines), list_area);

    // Footer
    if let Some(fa) = footer_area {
        let footer_text = format!(
            "Pods: {}  Running: {}",
            k8s.pod_count(),
            k8s.running_count(),
        );
        let footer_line = Line::from(Span::styled(
            footer_text,
            Style::default().fg(theme.text_secondary),
        ));
        frame.render_widget(Paragraph::new(footer_line), fa);
    }
}
