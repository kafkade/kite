# 🪁 Kite

A modern, cross-platform TUI system resource monitor written in Rust — inspired by [btop++](https://github.com/aristocratos/btop).

Kite gives you a real-time, interactive terminal dashboard for CPU, memory, disk, network, GPU, containers, and processes — with full keyboard/mouse control, customizable themes and layouts, configurable alerts, and remote monitoring over SSH.

> **Status**: 🚧 Early development — Phase 1 (core metrics MVP) in progress.

---

## Features (Planned)

### Phase 1 — Core Metrics *(in progress)*
- Real-time CPU monitoring (total + per-core, frequency, load averages)
- Memory & swap usage with historical graphs
- Disk I/O rates and filesystem usage
- Network interface traffic with auto-scaling graphs
- Interactive process table (sort, filter, search, tree view)
- Process management (kill, signal, renice)
- Configurable update interval and keybindings
- TOML-based configuration

### Phase 2 — Hardware & Theming
- GPU monitoring (NVIDIA, AMD, Intel, Apple Silicon)
- Hardware sensors (CPU/GPU/disk temperature, fan speed, voltage)
- Battery status and health
- Full theming engine (ship 10+ themes: Dracula, Catppuccin, Nord, etc.)
- Custom layouts and presets

### Phase 3 — Containers & Alerts
- Docker container monitoring and management
- Kubernetes pod/node monitoring
- Configurable threshold alerts with desktop notifications

### Phase 4 — Remote & Export
- SSH remote machine monitoring
- Prometheus metrics endpoint
- Metrics logging to JSON/CSV files
- Data replay mode for post-mortem analysis

### Phase 5 — Polish & Distribution
- Accessibility (screen reader, colorblind-safe themes)
- Internationalization
- Packaging for all platforms (deb, rpm, brew, winget, snap, etc.)
- Plugin/extension system

---

## Quick Start

### Prerequisites

- [Rust](https://rustup.rs/) 1.85+ (edition 2024)

### Build & Run

```bash
git clone https://github.com/your-org/kite.git
cd kite
cargo build --release
./target/release/kite
```

### Usage

```
kite [OPTIONS]

Options:
  -i, --interval <MS>  Update interval in milliseconds [default: 1000]
  -h, --help           Print help
  -V, --version        Print version
```

### Keyboard Shortcuts

| Key | Action |
|-----|--------|
| `q` | Quit |
| `Ctrl+C` | Quit |

> More keybindings will be added as features land.

---

## Configuration

Kite uses a TOML config file located at:

- **Linux/macOS**: `$XDG_CONFIG_HOME/kite/config.toml` or `~/.config/kite/config.toml`
- **Windows**: `%APPDATA%\kite\config.toml`

> Configuration system is coming in Stage 2. For now, use CLI flags.

---

## Architecture

```
src/
├── main.rs              # Entry point, CLI args (clap), async event loop
├── app.rs               # Application state machine
├── config/              # TOML config loading, keybindings
├── collector/           # Data collection (CPU, memory, disk, network, etc.)
│   └── platform/        # Platform-specific implementations
├── ui/
│   ├── layout.rs        # Layout engine and rendering
│   └── widgets/         # Individual panel widgets (cpu_box, mem_box, etc.)
├── input/               # Keyboard and mouse event handling
└── util/                # Ring buffer, unit formatting, error types
```

**Key design patterns:**

- **Collector trait**: Each data source implements `Collector` with async collection, polled by a scheduler at configurable intervals
- **Ring buffer history**: Fixed-size buffers store time-series data for graph rendering
- **Event-driven loop**: `tokio::select!` multiplexes input events, data ticks, and render ticks
- **RAII terminal guard**: Terminal is always restored on exit, even on panic
- **Platform abstraction**: `#[cfg(target_os)]` gates select platform-specific code at compile time

For the full specification, see [`docs/SPEC.md`](docs/SPEC.md).

---

## Contributing

### Getting Started

1. **Read the spec**: [`docs/SPEC.md`](docs/SPEC.md) contains the full project specification — architecture, module structure, design patterns, and all planned features across 5 phases.

2. **Check the plan**: [`PLAN.md`](PLAN.md) tracks implementation progress. The status table at the top shows what's done and what's next.

3. **Understand the stages**: Phase 1 is broken into 6 sequential stages. Pick up from the first `⬜ Not Started` stage — each stage's section in PLAN.md describes exactly what to build and the exit criteria.

4. **Set up your environment**:
   ```bash
   git clone https://github.com/your-org/kite.git
   cd kite
   cargo build
   cargo test
   ```

5. **Run the app** to see current state:
   ```bash
   cargo run
   cargo run -- --interval 500   # faster refresh
   ```

### Development Workflow

```bash
cargo build              # Build
cargo test               # Run all tests
cargo clippy             # Lint
cargo fmt                # Format
cargo run                # Launch the TUI
```

### Code Guidelines

- **Rust edition 2024**, target `clippy::pedantic` lint level
- `thiserror` for library/domain errors, `anyhow` at the binary boundary
- Prefer `tokio::sync::watch` channels for collector → UI data flow
- Each collector should be independently testable with mock data
- Document public APIs with `///` doc comments
- Keep functions small and modules focused

### Project Structure at a Glance

| File | Purpose |
|------|---------|
| `PLAN.md` | Implementation tracker — check here first |
| `docs/SPEC.md` | Full project specification (features, architecture, NFRs) |
| `docs/HANDOFF.md` | Context from the initial planning session |
| `Cargo.toml` | Dependencies and project metadata |

---

## Tech Stack

| Component | Technology |
|-----------|-----------|
| Language | Rust (edition 2024) |
| TUI | ratatui + crossterm |
| Async | tokio |
| System data | sysinfo |
| CLI | clap (derive) |
| Config | serde + toml |
| Errors | thiserror + anyhow |

---

## License

Dual-licensed under [MIT](LICENSE-MIT) or [Apache 2.0](LICENSE-APACHE) at your option.
