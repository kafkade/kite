# Kite — Copilot Instructions

## Build & Test

```bash
cargo build                       # Debug build
cargo build --release             # Release build
cargo test                        # Run all tests
cargo test ring_buffer            # Run tests matching a name
cargo test util::units::tests     # Run tests in a specific module
cargo clippy                      # Lint
cargo fmt                         # Format
cargo run                         # Launch TUI
cargo run -- --interval 500       # Launch with custom tick rate
```

## Architecture

Kite is an async TUI app built on `ratatui` + `crossterm` + `tokio`.

**Event loop** (`main.rs`): Uses `tokio::select!` to multiplex three streams — a data collection tick (user-configurable, default 1s), a render tick (fixed 250ms), and crossterm's `EventStream` for input. The render tick and data tick are intentionally decoupled so the UI stays responsive regardless of the collection interval.

**Terminal lifecycle** (`ui/mod.rs`): `TerminalGuard` is an RAII wrapper that enters alternate screen + raw mode on construction and restores on `Drop`. A separate panic hook (`install_panic_hook`) chains the previous hook and restores the terminal before printing the panic — this is a backup for the RAII guard.

**Collector pattern** (`collector/`): Each data source implements a `Collector` trait (CPU, memory, disk, network, process). Collectors produce snapshot values pushed into `RingBuffer<T>` (a fixed-capacity `VecDeque` wrapper in `util/ring_buffer.rs`) for time-series graph rendering.

**Data flow**: Collectors → `App` state (owned data) → `ui::layout::render()` reads `&App` to draw widgets. No channels yet — direct ownership. When remote/async collectors arrive, use `tokio::sync::watch` channels, not `Arc<RwLock<>>`.

**Widget pattern** (`ui/widgets/`): Each panel (cpu_box, mem_box, net_box, disk_box, proc_box) is a function that takes `&mut Frame`, a `Rect`, and the relevant data slice from `App`, then renders into that region using ratatui widgets.

**Overlays** (`ui/help.rs`, `ui/menu.rs`, `ui/dialog.rs`): Modal overlays render on top of the main layout. The `InputMode` enum in `app.rs` (Normal, Filtering, Help, Menu) determines how key events are routed.

**Input modes** (`input/keyboard.rs`): Key events are dispatched through mode-specific handlers (`handle_normal`, `handle_help`, `handle_menu`, `handle_filter`, `handle_dialog`). Dialog takes priority over all modes.

## Conventions

- **Error handling**: `thiserror` for domain error enums (`util/error.rs`), `anyhow::Result` at the binary boundary (`main.rs`). Don't use `unwrap()` in library code.
- **Graceful degradation**: If a system API is unavailable (sensor, GPU, container runtime), show "N/A" — never crash.
- **Input handling**: Key events are dispatched by `InputMode` in `input/keyboard.rs`. Each mode (Normal, Help, Menu, Filtering) has its own handler. When adding new keybindings, match on `KeyCode` variants; modifier keys use `key.modifiers.contains(KeyModifiers::CONTROL)`.
- **App state mutations**: Go through `App` methods — don't reach into fields directly from input handlers or UI code.
- **Platform-specific code**: Use `#[cfg(target_os = "...")]` at the module level in `collector/platform/`. Implement the same trait interface for each platform.

## Git Policy

**Never execute Git commands that modify history or submit code.** This includes `git commit`, `git push`, `git rebase`, `git merge`, `git reset`, `git cherry-pick`, `git revert`, and `git tag`. Read-only commands like `git status`, `git diff`, `git log`, and `git branch` are fine. A human must always review and commit code themselves.

## Key References

- `PLAN.md` — Implementation tracker with stage status.
- `docs/SPEC.md` — Full project specification: features, architecture, module structure, NFRs, design decisions.
- `docs/RELEASING.md` — Release process, changelog maintenance, versioning policy.
- `docs/HANDOFF.md` — Context from initial planning sessions.
