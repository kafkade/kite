mod alert;
mod app;
mod collector;
mod config;
pub mod export;
mod input;
mod ui;
mod util;

use std::time::Duration;

use anyhow::Result;
use clap::Parser;
use crossterm::event::{Event, EventStream};
use futures::StreamExt;

use app::App;
use export::log::MetricsLogger;
use input::AppEvent;

#[cfg(feature = "prometheus")]
use std::sync::Arc;
#[cfg(feature = "prometheus")]
use tokio::sync::RwLock;

/// Kite — a modern cross-platform TUI system resource monitor
#[derive(Parser, Debug)]
#[command(version, about)]
struct Cli {
    /// Update interval in milliseconds (min: 100)
    #[arg(short, long)]
    interval: Option<u64>,

    /// Generate a default config file and exit
    #[arg(long)]
    generate_config: bool,

    /// Theme name (e.g., dracula, nord, catppuccin-mocha)
    #[arg(long)]
    theme: Option<String>,

    /// Layout preset (default, minimal, full, server, laptop, gpu-focus)
    #[arg(long)]
    layout: Option<String>,

    /// Replay a metrics log file instead of live monitoring
    #[arg(long)]
    replay: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    if cli.generate_config {
        let path = config::generate_default()?;
        println!("Default config written to: {}", path.display());
        return Ok(());
    }

    // Load config: file defaults → CLI overrides
    let mut cfg = config::load()?;
    config::apply_cli_overrides(
        &mut cfg,
        cli.interval,
        cli.theme.as_deref(),
        cli.layout.as_deref(),
    );

    ui::install_panic_hook();

    let mut guard = ui::TerminalGuard::new()?;
    let terminal = guard.terminal_mut();

    let interval_ms = cfg.update_interval_ms;
    let mut logger = MetricsLogger::new(&cfg.logging);

    // Prometheus metrics exporter (feature-gated)
    #[cfg(feature = "prometheus")]
    let prom_snapshot = Arc::new(RwLock::new(export::prometheus::MetricsSnapshot::default()));
    #[cfg(feature = "prometheus")]
    let (prom_shutdown_tx, prom_shutdown_rx) = tokio::sync::watch::channel(false);
    #[cfg(feature = "prometheus")]
    let prom_enabled = cfg.prometheus.enabled;

    #[cfg(feature = "prometheus")]
    let prom_config = cfg.prometheus.clone();

    let mut app = App::new(cfg);

    // Load replay file if specified
    if let Some(ref replay_path) = cli.replay {
        let state = export::replay::ReplayState::load(replay_path)
            .map_err(|e| anyhow::anyhow!("Failed to load replay file: {e}"))?;
        app.replay = Some(state);
        app.apply_current_replay();
    }

    // Spawn Prometheus server if enabled
    #[cfg(feature = "prometheus")]
    {
        if prom_enabled {
            let snap = Arc::clone(&prom_snapshot);
            tokio::spawn(export::prometheus::serve(
                prom_config,
                snap,
                prom_shutdown_rx,
            ));
        }
    }

    // Initial draw before entering the loop
    terminal.draw(|frame| ui::layout::render(frame, &app))?;

    let mut data_tick = tokio::time::interval(Duration::from_millis(interval_ms));
    let mut render_tick = tokio::time::interval(Duration::from_millis(250));
    let mut event_stream = EventStream::new();

    loop {
        let event = tokio::select! {
            _ = data_tick.tick() => AppEvent::Tick,
            _ = render_tick.tick() => AppEvent::RenderTick,
            maybe_event = event_stream.next() => {
                match maybe_event {
                    Some(Ok(Event::Key(key))) => AppEvent::Key(key),
                    Some(Ok(Event::Mouse(mouse))) => AppEvent::Mouse(mouse),
                    Some(Ok(Event::Resize(w, h))) => AppEvent::Resize(w, h),
                    Some(Err(_)) | None => break,
                    _ => continue,
                }
            }
        };

        match event {
            AppEvent::Key(key) => input::keyboard::handle_key_event(&mut app, key),
            AppEvent::Resize(w, h) => app.on_resize(w, h),
            AppEvent::Tick => {
                if app.is_replay_mode() {
                    app.replay_auto_advance();
                } else {
                    app.collect_all();
                    if let Some(ref mut log) = logger {
                        log.log_tick(&app);
                    }
                    #[cfg(feature = "prometheus")]
                    {
                        if prom_enabled {
                            let snap = export::prometheus::collect_snapshot(&app);
                            *prom_snapshot.write().await = snap;
                        }
                    }
                    if app.alerts.bell_pending() {
                        print!("\x07");
                        app.alerts.clear_bell();
                    }
                }
            }
            AppEvent::RenderTick | AppEvent::Mouse(_) => {}
        }

        if !app.is_running() {
            break;
        }

        terminal.draw(|frame| ui::layout::render(frame, &app))?;
    }

    // Signal Prometheus server to shut down
    #[cfg(feature = "prometheus")]
    {
        let _ = prom_shutdown_tx.send(true);
    }

    // TerminalGuard::drop handles cleanup
    Ok(())
}
