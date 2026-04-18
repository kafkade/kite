use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::Paragraph,
};

use crate::app::{App, InputMode};
use crate::ui::widgets::{cpu_box, disk_box, gpu_box, mem_box, net_box, proc_box, sensor_box};

/// Render the full application layout.
pub fn render(frame: &mut Frame, app: &App) {
    let size = frame.area();

    // Split into: main content area + status bar (1 line)
    let outer = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(3),    // main content
            Constraint::Length(1), // status bar
        ])
        .split(size);

    render_main_area(frame, outer[0], app);
    render_status_bar(frame, outer[1], app);

    // Render overlays on top of everything (highest priority last)
    match app.input_mode {
        InputMode::Help => crate::ui::help::render(frame),
        InputMode::Menu => {
            if let Some(ref menu) = app.menu {
                crate::ui::menu::render(frame, menu);
            }
        }
        _ => {}
    }

    // Dialog always renders on top of everything else
    if let Some(ref dialog) = app.dialog {
        crate::ui::dialog::render(frame, dialog);
    }
}

/// Render the main content area with adaptive layout.
/// When GPU or Sensors panels are visible, a hardware row is added.
/// Otherwise the original 3-row layout is used.
fn render_main_area(frame: &mut Frame, area: Rect, app: &App) {
    let panels = &app.config().panels;
    let show_hw_row = panels.gpu || panels.sensors;

    let mut constraints = Vec::new();
    let mut row_ids: Vec<&str> = Vec::new();

    // Row: CPU + Memory
    if panels.cpu || panels.memory {
        constraints.push(if show_hw_row {
            Constraint::Percentage(25)
        } else {
            Constraint::Percentage(30)
        });
        row_ids.push("cpu_mem");
    }

    // Row: Network + Disk
    if panels.network || panels.disk {
        constraints.push(if show_hw_row {
            Constraint::Percentage(20)
        } else {
            Constraint::Percentage(25)
        });
        row_ids.push("net_disk");
    }

    // Row: GPU + Sensors (only if at least one is enabled)
    if show_hw_row {
        constraints.push(Constraint::Percentage(20));
        row_ids.push("hw");
    }

    // Row: Processes (always takes remaining space)
    if panels.processes {
        constraints.push(Constraint::Min(5));
        row_ids.push("proc");
    }

    if constraints.is_empty() {
        return;
    }

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(area);

    for (i, &row_id) in row_ids.iter().enumerate() {
        match row_id {
            "cpu_mem" => {
                let cols = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                    .split(rows[i]);
                if panels.cpu {
                    cpu_box::render(frame, cols[0], &app.cpu);
                }
                if panels.memory {
                    mem_box::render(frame, cols[1], &app.mem);
                }
            }
            "net_disk" => {
                let cols = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                    .split(rows[i]);
                if panels.network {
                    net_box::render(frame, cols[0], &app.net);
                }
                if panels.disk {
                    disk_box::render(frame, cols[1], &app.disk);
                }
            }
            "hw" => {
                let cols = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                    .split(rows[i]);
                if panels.gpu {
                    gpu_box::render(frame, cols[0], &app.gpu);
                }
                if panels.sensors {
                    sensor_box::render(frame, cols[1], &app.sensor);
                }
            }
            "proc" => {
                proc_box::render(frame, rows[i], &app.proc_collector, &app.proc_widget);
            }
            _ => {}
        }
    }
}

/// Render the bottom status bar.
fn render_status_bar(frame: &mut Frame, area: Rect, app: &App) {
    let now = chrono::Local::now().format("%H:%M:%S").to_string();
    let uptime_str = crate::util::units::format_duration(app.uptime_secs());

    let mode_indicator = match app.input_mode {
        InputMode::Normal => "",
        InputMode::Filtering => " [FILTER]",
        InputMode::Help => " [HELP]",
        InputMode::Menu => " [MENU]",
    };

    let left = format!(
        " Kite v{} │ {}{} ",
        env!("CARGO_PKG_VERSION"),
        app.hostname(),
        mode_indicator,
    );
    let right = format!(
        " ? help │ {} │ up {} │ ↻ {}ms ",
        now,
        uptime_str,
        app.update_interval_ms()
    );

    let padding = area.width.saturating_sub((left.len() + right.len()) as u16);

    let bar = Line::from(vec![
        Span::styled(&left, Style::default().fg(Color::Cyan)),
        Span::raw(" ".repeat(padding as usize)),
        Span::styled(&right, Style::default().fg(Color::DarkGray)),
    ]);

    let bar_widget =
        Paragraph::new(bar).style(Style::default().bg(Color::DarkGray).fg(Color::White));

    frame.render_widget(bar_widget, area);
}
