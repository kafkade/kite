mod app;
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
    /// Update interval in milliseconds
    #[arg(short, long, default_value_t = 1000)]
    interval: u64,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    ui::install_panic_hook();

    let mut guard = ui::TerminalGuard::new()?;
    let terminal = guard.terminal_mut();
    let mut app = App::new(cli.interval);

    // Initial draw before entering the loop
    terminal.draw(|frame| ui::layout::render(frame, &app))?;

    let mut data_tick = tokio::time::interval(Duration::from_millis(cli.interval));
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
                // Data collection will happen here in Stage 3+
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

