use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

use crate::collector::docker::{ContainerInfo, ContainerState, DockerCollector};
use crate::ui::theme::Theme;
use crate::util::units::format_bytes;

// Column widths
const COL_NAME: u16 = 16;
const COL_IMAGE: u16 = 18;
const COL_STATUS: u16 = 12;
const COL_CPU: u16 = 7;
const COL_MEM: u16 = 14;
const COL_NET: u16 = 14;

pub struct ContainerWidget {
    pub selected_index: usize,
    pub scroll_offset: usize,
}

#[allow(dead_code)]
impl ContainerWidget {
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

    pub fn selected_container<'a>(
        &self,
        containers: &'a [ContainerInfo],
    ) -> Option<&'a ContainerInfo> {
        containers.get(self.selected_index)
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

fn state_color(state: ContainerState, theme: &Theme) -> Color {
    match state {
        ContainerState::Running => theme.good,
        ContainerState::Paused => theme.warning,
        ContainerState::Exited | ContainerState::Dead => theme.critical,
        _ => theme.text_secondary,
    }
}

fn cpu_color(cpu: f64, theme: &Theme) -> Color {
    if cpu > 75.0 {
        theme.critical
    } else if cpu >= 25.0 {
        theme.warning
    } else {
        theme.good
    }
}

fn build_header_line(width: u16, theme: &Theme) -> Line<'static> {
    let blkio_width =
        width.saturating_sub(COL_NAME + COL_IMAGE + COL_STATUS + COL_CPU + COL_MEM + COL_NET);
    let hdr_style = Style::default()
        .fg(theme.text_secondary)
        .add_modifier(Modifier::BOLD);

    let text = format!(
        "{:<name_w$}{:<img_w$}{:<st_w$}{:>cpu_w$}{:>mem_w$}{:>net_w$}{:>blk_w$}",
        "Name",
        "Image",
        "Status",
        "CPU%",
        "Memory",
        "Net I/O",
        "Block I/O",
        name_w = COL_NAME as usize,
        img_w = COL_IMAGE as usize,
        st_w = COL_STATUS as usize,
        cpu_w = COL_CPU as usize,
        mem_w = COL_MEM as usize,
        net_w = COL_NET as usize,
        blk_w = blkio_width as usize,
    );

    Line::from(Span::styled(text, hdr_style))
}

fn build_row_line(
    container: &ContainerInfo,
    width: u16,
    selected: bool,
    theme: &Theme,
) -> Line<'static> {
    let blkio_width =
        width.saturating_sub(COL_NAME + COL_IMAGE + COL_STATUS + COL_CPU + COL_MEM + COL_NET);

    let base_style = if selected {
        Style::default()
            .bg(theme.selected_bg)
            .fg(theme.selected_fg)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(theme.text_primary)
    };

    let name_text = truncate(&container.name, COL_NAME as usize);
    let image_text = truncate(&container.image, COL_IMAGE as usize);
    let status_text = truncate(&container.state.to_string(), COL_STATUS as usize);

    let cpu_text = format!("{:.1}", container.cpu_percent);
    let mem_text = if container.memory_limit > 0 {
        format!(
            "{}/{}",
            format_bytes(container.memory_used),
            format_bytes(container.memory_limit)
        )
    } else if container.memory_used > 0 {
        format_bytes(container.memory_used)
    } else {
        "-".to_string()
    };
    let net_text = if container.net_rx > 0 || container.net_tx > 0 {
        format!(
            "{}/{}",
            format_bytes(container.net_rx),
            format_bytes(container.net_tx)
        )
    } else {
        "-".to_string()
    };
    let blk_text = if container.block_read > 0 || container.block_write > 0 {
        format!(
            "{}/{}",
            format_bytes(container.block_read),
            format_bytes(container.block_write)
        )
    } else {
        "-".to_string()
    };

    let status_color = if selected {
        theme.selected_fg
    } else {
        state_color(container.state, theme)
    };

    let cpu_fg = if selected {
        theme.selected_fg
    } else {
        cpu_color(container.cpu_percent, theme)
    };

    let image_style = if selected {
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
            format!("{:<w$}", image_text, w = COL_IMAGE as usize),
            image_style,
        ),
        Span::styled(
            format!("{:<w$}", status_text, w = COL_STATUS as usize),
            base_style.fg(status_color),
        ),
        Span::styled(
            format!("{:>w$}", cpu_text, w = COL_CPU as usize),
            base_style.fg(cpu_fg),
        ),
        Span::styled(
            format!(
                "{:>w$}",
                truncate(&mem_text, COL_MEM as usize),
                w = COL_MEM as usize
            ),
            base_style,
        ),
        Span::styled(
            format!(
                "{:>w$}",
                truncate(&net_text, COL_NET as usize),
                w = COL_NET as usize
            ),
            base_style,
        ),
        Span::styled(
            format!(
                "{:>w$}",
                truncate(&blk_text, blkio_width as usize),
                w = blkio_width as usize
            ),
            base_style,
        ),
    ];

    Line::from(spans)
}

#[allow(dead_code)]
pub fn render(
    frame: &mut Frame,
    area: Rect,
    docker: &DockerCollector,
    widget: &ContainerWidget,
    theme: &Theme,
) {
    let outer_block = Block::default()
        .title(" Docker ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.docker_border));

    let inner = outer_block.inner(area);
    frame.render_widget(outer_block, area);

    if inner.width < 4 || inner.height < 3 {
        return;
    }

    // Docker not available
    if !docker.has_docker() {
        let msg = Paragraph::new(Line::from(Span::styled(
            "Docker not available",
            Style::default().fg(theme.text_secondary),
        )))
        .alignment(ratatui::layout::Alignment::Center);
        frame.render_widget(msg, inner);
        return;
    }

    let containers = docker.containers();

    // No containers
    if containers.is_empty() {
        let msg = Paragraph::new(Line::from(Span::styled(
            "No containers",
            Style::default().fg(theme.text_secondary),
        )))
        .alignment(ratatui::layout::Alignment::Center);
        frame.render_widget(msg, inner);
        return;
    }

    // Layout: header(1) + container list + footer(1)
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

    // Container rows
    let visible_height = list_height as usize;
    let mut widget_state = ContainerWidget {
        selected_index: widget.selected_index,
        scroll_offset: widget.scroll_offset,
    };
    widget_state.ensure_visible(visible_height);

    let end = (widget_state.scroll_offset + visible_height).min(containers.len());
    let start = widget_state.scroll_offset.min(end);

    let mut lines: Vec<Line> = Vec::new();
    for (i, container) in containers.iter().enumerate().skip(start).take(end - start) {
        let selected = i == widget_state.selected_index;
        lines.push(build_row_line(container, inner.width, selected, theme));
    }

    frame.render_widget(Paragraph::new(lines), list_area);

    // Footer
    if let Some(fa) = footer_area {
        let footer_text = format!(
            "Containers: {}  Running: {}  │  s:start  S:stop  R:restart",
            docker.container_count(),
            docker.running_count(),
        );
        let footer_line = Line::from(Span::styled(
            footer_text,
            Style::default().fg(theme.text_secondary),
        ));
        frame.render_widget(Paragraph::new(footer_line), fa);
    }
}
