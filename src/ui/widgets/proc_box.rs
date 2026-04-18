use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

use crate::collector::process::{ProcessCollector, ProcessInfo, SortColumn, SortOrder};
use crate::ui::theme::Theme;
use crate::util::units::format_bytes;

pub struct ProcessWidget {
    pub selected_index: usize,
    pub scroll_offset: usize,
    pub show_tree: bool,
    pub filter_input: String,
    pub is_filtering: bool,
}

#[allow(dead_code)]
impl ProcessWidget {
    pub fn new() -> Self {
        Self {
            selected_index: 0,
            scroll_offset: 0,
            show_tree: false,
            filter_input: String::new(),
            is_filtering: false,
        }
    }

    pub fn scroll_up(&mut self) {
        self.selected_index = self.selected_index.saturating_sub(1);
    }

    pub fn scroll_down(&mut self, total_items: usize) {
        if total_items > 0 && self.selected_index < total_items - 1 {
            self.selected_index += 1;
        }
    }

    pub fn page_up(&mut self) {
        self.selected_index = self
            .selected_index
            .saturating_sub(self.visible_height_hint());
    }

    pub fn page_down(&mut self, total_items: usize, visible_height: usize) {
        if total_items == 0 {
            return;
        }
        self.selected_index = (self.selected_index + visible_height).min(total_items - 1);
    }

    pub fn toggle_tree(&mut self) {
        self.show_tree = !self.show_tree;
    }

    pub fn selected_pid(&self, processes: &[ProcessInfo]) -> Option<u32> {
        processes.get(self.selected_index).map(|p| p.pid)
    }

    pub fn ensure_visible(&mut self, visible_height: usize) {
        if visible_height == 0 {
            return;
        }
        if self.selected_index < self.scroll_offset {
            self.scroll_offset = self.selected_index;
        } else if self.selected_index >= self.scroll_offset + visible_height {
            self.scroll_offset = self.selected_index - visible_height + 1;
        }
    }

    pub fn start_filter(&mut self) {
        self.is_filtering = true;
    }

    pub fn stop_filter(&mut self) {
        self.is_filtering = false;
    }

    pub fn append_filter_char(&mut self, c: char) {
        self.filter_input.push(c);
    }

    pub fn backspace_filter(&mut self) {
        self.filter_input.pop();
    }

    // Fallback hint when visible_height isn't known yet (used by page_up)
    fn visible_height_hint(&self) -> usize {
        20
    }
}

// Column widths: PID(7) Name(20) User(10) CPU%(7) MEM%(7) MEM(9) State(8) Threads(4) Command(rest)
const COL_PID: u16 = 7;
const COL_NAME: u16 = 20;
const COL_USER: u16 = 10;
const COL_CPU: u16 = 7;
const COL_MEM_PCT: u16 = 7;
const COL_MEM: u16 = 9;
const COL_STATE: u16 = 8;
const COL_THREADS: u16 = 4;

fn sort_indicator(col: SortColumn, active: SortColumn, order: SortOrder) -> &'static str {
    if col == active {
        match order {
            SortOrder::Ascending => "▲",
            SortOrder::Descending => "▼",
        }
    } else {
        ""
    }
}

fn sort_column_label(col: SortColumn) -> &'static str {
    match col {
        SortColumn::Pid => "PID",
        SortColumn::Name => "Name",
        SortColumn::User => "User",
        SortColumn::Cpu => "CPU%",
        SortColumn::Memory => "MEM%",
        SortColumn::Status => "State",
        SortColumn::Threads => "Threads",
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

fn cpu_color(cpu: f32, theme: &Theme) -> Color {
    if cpu > 75.0 {
        theme.critical
    } else if cpu >= 25.0 {
        theme.warning
    } else {
        theme.good
    }
}

fn build_header_line(
    sort_col: SortColumn,
    sort_ord: SortOrder,
    width: u16,
    theme: &Theme,
) -> Line<'static> {
    let cmd_width = width.saturating_sub(
        COL_PID + COL_NAME + COL_USER + COL_CPU + COL_MEM_PCT + COL_MEM + COL_STATE + COL_THREADS,
    );
    let hdr_style = Style::default()
        .fg(theme.text_primary)
        .add_modifier(Modifier::BOLD);

    let cols: Vec<(String, u16, bool)> = vec![
        (
            format!(
                "{:>w$}",
                format!("PID{}", sort_indicator(SortColumn::Pid, sort_col, sort_ord)),
                w = COL_PID as usize
            ),
            COL_PID,
            true,
        ),
        (
            format!(
                "{:<w$}",
                format!(
                    "Name{}",
                    sort_indicator(SortColumn::Name, sort_col, sort_ord)
                ),
                w = COL_NAME as usize
            ),
            COL_NAME,
            false,
        ),
        (
            format!(
                "{:<w$}",
                format!(
                    "User{}",
                    sort_indicator(SortColumn::User, sort_col, sort_ord)
                ),
                w = COL_USER as usize
            ),
            COL_USER,
            false,
        ),
        (
            format!(
                "{:>w$}",
                format!(
                    "CPU%{}",
                    sort_indicator(SortColumn::Cpu, sort_col, sort_ord)
                ),
                w = COL_CPU as usize
            ),
            COL_CPU,
            true,
        ),
        (
            format!(
                "{:>w$}",
                format!(
                    "MEM%{}",
                    sort_indicator(SortColumn::Memory, sort_col, sort_ord)
                ),
                w = COL_MEM_PCT as usize
            ),
            COL_MEM_PCT,
            true,
        ),
        (
            format!("{:>w$}", "MEM", w = COL_MEM as usize),
            COL_MEM,
            true,
        ),
        (
            format!(
                "{:<w$}",
                format!(
                    "State{}",
                    sort_indicator(SortColumn::Status, sort_col, sort_ord)
                ),
                w = COL_STATE as usize
            ),
            COL_STATE,
            false,
        ),
        (
            format!(
                "{:>w$}",
                format!(
                    "Thr{}",
                    sort_indicator(SortColumn::Threads, sort_col, sort_ord)
                ),
                w = COL_THREADS as usize
            ),
            COL_THREADS,
            true,
        ),
        (
            format!("{:<w$}", "Command", w = cmd_width as usize),
            cmd_width,
            false,
        ),
    ];

    let text: String = cols
        .into_iter()
        .map(|(s, _, _)| s)
        .collect::<Vec<_>>()
        .join("");
    Line::from(Span::styled(text, hdr_style))
}

#[allow(clippy::too_many_arguments)]
fn build_row_spans(
    pid: u32,
    name: &str,
    user: &str,
    cpu: f32,
    mem_pct: f32,
    mem_bytes: u64,
    status: &str,
    threads: Option<u32>,
    command: &str,
    width: u16,
    selected: bool,
    tree_prefix: &str,
    theme: &Theme,
) -> Line<'static> {
    let cmd_width = width.saturating_sub(
        COL_PID + COL_NAME + COL_USER + COL_CPU + COL_MEM_PCT + COL_MEM + COL_STATE + COL_THREADS,
    ) as usize;

    let display_name = if tree_prefix.is_empty() {
        truncate(name, COL_NAME as usize)
    } else {
        let prefixed = format!("{}{}", tree_prefix, name);
        truncate(&prefixed, COL_NAME as usize)
    };

    let base_style = if selected {
        Style::default()
            .bg(theme.selected_bg)
            .fg(theme.selected_fg)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };

    let cpu_style = base_style.fg(cpu_color(cpu, theme));

    let thr_str = threads.map_or("-".to_string(), |t| t.to_string());
    let mem_str = format_bytes(mem_bytes);

    let spans = vec![
        Span::styled(format!("{:>w$}", pid, w = COL_PID as usize), base_style),
        Span::styled(
            format!("{:<w$}", display_name, w = COL_NAME as usize),
            base_style,
        ),
        Span::styled(
            format!(
                "{:<w$}",
                truncate(user, COL_USER as usize),
                w = COL_USER as usize
            ),
            base_style,
        ),
        Span::styled(format!("{:>w$.1}", cpu, w = COL_CPU as usize), cpu_style),
        Span::styled(
            format!("{:>w$.1}", mem_pct, w = COL_MEM_PCT as usize),
            base_style,
        ),
        Span::styled(format!("{:>w$}", mem_str, w = COL_MEM as usize), base_style),
        Span::styled(
            format!(
                "{:<w$}",
                truncate(status, COL_STATE as usize),
                w = COL_STATE as usize
            ),
            base_style,
        ),
        Span::styled(
            format!("{:>w$}", thr_str, w = COL_THREADS as usize),
            base_style,
        ),
        Span::styled(
            format!("{:<w$}", truncate(command, cmd_width), w = cmd_width),
            base_style,
        ),
    ];

    Line::from(spans)
}

pub fn render(
    frame: &mut Frame,
    area: Rect,
    proc_collector: &ProcessCollector,
    widget: &ProcessWidget,
    theme: &Theme,
) {
    let title = if proc_collector.is_paused() {
        " Processes [PAUSED] "
    } else {
        " Processes "
    };

    let outer_block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.accent));

    let inner = outer_block.inner(area);
    frame.render_widget(outer_block, area);

    if inner.width < 10 || inner.height < 4 {
        return;
    }

    let show_filter_bar = widget.is_filtering || !widget.filter_input.is_empty();

    // Layout: header(1) + optional filter(1) + process list + status(1)
    let constraints = if show_filter_bar {
        vec![
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Min(1),
            Constraint::Length(1),
        ]
    } else {
        vec![
            Constraint::Length(1),
            Constraint::Min(1),
            Constraint::Length(1),
        ]
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(inner);

    let (header_area, filter_area, list_area, status_area) = if show_filter_bar {
        (chunks[0], Some(chunks[1]), chunks[2], chunks[3])
    } else {
        (chunks[0], None, chunks[1], chunks[2])
    };

    // Header
    let header = build_header_line(
        proc_collector.sort_column(),
        proc_collector.sort_order(),
        inner.width,
        theme,
    );
    frame.render_widget(Paragraph::new(header), header_area);

    // Filter bar
    if let Some(fa) = filter_area {
        let filter_text = if widget.is_filtering {
            format!("Filter: {}_", widget.filter_input)
        } else {
            let count = proc_collector.processes().len();
            format!("Filter: {} ({} results)", widget.filter_input, count)
        };
        let filter_line = Line::from(Span::styled(filter_text, Style::default().fg(theme.accent)));
        frame.render_widget(Paragraph::new(filter_line), fa);
    }

    // Process list
    let visible_height = list_area.height as usize;
    let processes = proc_collector.processes();
    let total_items = processes.len();

    let mut lines: Vec<Line> = Vec::new();

    if widget.show_tree {
        let tree_nodes = proc_collector.tree();
        let end = (widget.scroll_offset + visible_height).min(tree_nodes.len());
        let start = widget.scroll_offset.min(end);
        for (i, node) in tree_nodes.iter().enumerate().skip(start).take(end - start) {
            let prefix = "  ".repeat(node.depth);
            let selected = i == widget.selected_index;
            let p = &node.process;
            lines.push(build_row_spans(
                p.pid,
                &p.name,
                &p.user,
                p.cpu_percent,
                p.mem_percent,
                p.mem_bytes,
                &p.status,
                p.threads,
                &p.command,
                inner.width,
                selected,
                &prefix,
                theme,
            ));
        }
    } else {
        let end = (widget.scroll_offset + visible_height).min(total_items);
        let start = widget.scroll_offset.min(end);
        for (i, p) in processes.iter().enumerate().skip(start).take(end - start) {
            let selected = i == widget.selected_index;
            lines.push(build_row_spans(
                p.pid,
                &p.name,
                &p.user,
                p.cpu_percent,
                p.mem_percent,
                p.mem_bytes,
                &p.status,
                p.threads,
                &p.command,
                inner.width,
                selected,
                "",
                theme,
            ));
        }
    }

    frame.render_widget(Paragraph::new(lines), list_area);

    // Status line
    let sort_label = sort_column_label(proc_collector.sort_column());
    let sort_arrow = match proc_collector.sort_order() {
        SortOrder::Ascending => "▲",
        SortOrder::Descending => "▼",
    };
    let status_text = format!(
        "Total: {}  Showing: {}  │  Sort: {} {}  │  Space: pause  T: tree",
        proc_collector.all_process_count(),
        total_items,
        sort_label,
        sort_arrow,
    );
    let status_line = Line::from(Span::styled(
        status_text,
        Style::default().fg(theme.text_secondary),
    ));
    frame.render_widget(Paragraph::new(status_line), status_area);
}
