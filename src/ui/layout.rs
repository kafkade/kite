use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::Style,
    text::{Line, Span},
    widgets::Paragraph,
};

use crate::app::{App, InputMode};
use crate::ui::theme::Theme;
use crate::ui::widgets::{
    container_box, cpu_box, disk_box, gpu_box, k8s_box, mem_box, net_box, proc_box, sensor_box,
};

/// Render the full application layout.
pub fn render(frame: &mut Frame, app: &App) {
    let size = frame.area();
    let theme = &app.theme;

    // Split into: main content area + status bar (1 line)
    let outer = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(3),    // main content
            Constraint::Length(1), // status bar
        ])
        .split(size);

    render_main_area(frame, outer[0], app, theme);
    render_status_bar(frame, outer[1], app, theme);

    // Render overlays on top of everything (highest priority last)
    match app.input_mode {
        InputMode::Help => crate::ui::help::render(frame, theme),
        InputMode::Menu => {
            if let Some(ref menu) = app.menu {
                crate::ui::menu::render(frame, menu, theme);
            }
        }
        _ => {}
    }

    // Dialog always renders on top of everything else
    if let Some(ref dialog) = app.dialog {
        crate::ui::dialog::render(frame, dialog, theme);
    }
}

/// Render the main content area with adaptive layout.
/// When GPU or Sensors panels are visible, a hardware row is added.
/// Otherwise the original 3-row layout is used.
fn render_main_area(frame: &mut Frame, area: Rect, app: &App, theme: &Theme) {
    let panels = &app.config().panels;
    let show_hw_row = panels.gpu || panels.sensors;
    let show_container_row = panels.docker || panels.k8s;

    let mut constraints = Vec::new();
    let mut row_ids: Vec<&str> = Vec::new();

    // Row: CPU + Memory
    if panels.cpu || panels.memory {
        constraints.push(if show_hw_row || show_container_row {
            Constraint::Percentage(25)
        } else {
            Constraint::Percentage(30)
        });
        row_ids.push("cpu_mem");
    }

    // Row: Network + Disk
    if panels.network || panels.disk {
        constraints.push(if show_hw_row || show_container_row {
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

    // Row: Containers (Docker and/or Kubernetes)
    if show_container_row {
        if panels.docker && panels.k8s {
            constraints.push(Constraint::Percentage(20));
            row_ids.push("containers_both");
        } else {
            constraints.push(Constraint::Percentage(20));
            row_ids.push("containers_single");
        }
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
                    cpu_box::render(frame, cols[0], &app.cpu, theme);
                }
                if panels.memory {
                    mem_box::render(frame, cols[1], &app.mem, theme);
                }
            }
            "net_disk" => {
                let cols = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                    .split(rows[i]);
                if panels.network {
                    net_box::render(frame, cols[0], &app.net, theme);
                }
                if panels.disk {
                    disk_box::render(frame, cols[1], &app.disk, theme);
                }
            }
            "hw" => {
                let cols = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                    .split(rows[i]);
                if panels.gpu {
                    gpu_box::render(frame, cols[0], &app.gpu, theme);
                }
                if panels.sensors {
                    sensor_box::render(frame, cols[1], &app.sensor, theme);
                }
            }
            "docker" | "containers_both" => {
                let cols = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                    .split(rows[i]);
                container_box::render(frame, cols[0], &app.docker, &app.container_widget, theme);
                k8s_box::render(frame, cols[1], &app.k8s, &app.k8s_widget, theme);
            }
            "containers_single" => {
                if panels.docker {
                    container_box::render(
                        frame,
                        rows[i],
                        &app.docker,
                        &app.container_widget,
                        theme,
                    );
                }
                if panels.k8s {
                    k8s_box::render(frame, rows[i], &app.k8s, &app.k8s_widget, theme);
                }
            }
            "proc" => {
                proc_box::render(frame, rows[i], &app.proc_collector, &app.proc_widget, theme);
            }
            _ => {}
        }
    }
}

/// Render the bottom status bar.
fn render_status_bar(frame: &mut Frame, area: Rect, app: &App, theme: &Theme) {
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

    // Battery info (empty string if no battery)
    let battery_str = app.battery.format_compact();
    let battery_section = if battery_str.is_empty() {
        String::new()
    } else {
        format!(" │ {} ", battery_str)
    };

    // Alert indicator
    let alert_str = app.alerts.format_indicator();
    let alert_section = if alert_str.is_empty() {
        String::new()
    } else {
        format!(" {} ", alert_str)
    };

    let alert_color = match app.alerts.highest_severity() {
        Some(crate::alert::Severity::Critical) => theme.critical,
        Some(crate::alert::Severity::Warning) => theme.warning,
        _ => theme.accent,
    };

    let right = format!(
        " {} │ {} │ up {} │ ↻ {}ms ",
        app.theme.name,
        now,
        uptime_str,
        app.update_interval_ms()
    );

    let padding = area.width.saturating_sub(
        (left.len() + battery_section.len() + alert_section.len() + right.len()) as u16,
    );

    let bar = Line::from(vec![
        Span::styled(&left, Style::default().fg(theme.status_bar_accent)),
        Span::styled(&battery_section, Style::default().fg(theme.good)),
        Span::styled(&alert_section, Style::default().fg(alert_color)),
        Span::raw(" ".repeat(padding as usize)),
        Span::styled(&right, Style::default().fg(theme.text_secondary)),
    ]);

    let bar_widget = Paragraph::new(bar).style(
        Style::default()
            .bg(theme.status_bar_bg)
            .fg(theme.status_bar_fg),
    );

    frame.render_widget(bar_widget, area);
}
