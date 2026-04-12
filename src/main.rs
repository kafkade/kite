mod app;
mod collector;
mod config;
mod input;
mod ui;
mod util;

use std::time::Duration;

use anyhow::Result;
use clap::Parser;
use crossterm::event::{Event, EventStream};
use futures::StreamExt;

use app::App;
use input::AppEvent;

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
    config::apply_cli_overrides(&mut cfg, cli.interval);

    ui::install_panic_hook();

    let mut guard = ui::TerminalGuard::new()?;
    let terminal = guard.terminal_mut();

    let interval_ms = cfg.update_interval_ms;
    let mut app = App::new(cfg);

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
                app.collect_all();
            }
            AppEvent::RenderTick | AppEvent::Mouse(_) => {}
        }

        if !app.is_running() {
            break;
        }

        terminal.draw(|frame| ui::layout::render(frame, &app))?;
    }

    // TerminalGuard::drop handles cleanup
    Ok(())
}
