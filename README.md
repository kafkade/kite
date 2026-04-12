# 🪁 Kite

[![CI](https://github.com/kafkade/kite/actions/workflows/ci.yml/badge.svg)](https://github.com/kafkade/kite/actions/workflows/ci.yml)
[![License: MIT OR Apache-2.0](https://img.shields.io/crates/l/kite)](LICENSE-MIT)
[![Rust](https://img.shields.io/badge/rust-1.85%2B-orange.svg)](https://www.rust-lang.org/)

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
git clone https://github.com/kafkade/kite.git
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

We welcome contributions! Please see our [Contributing Guide](CONTRIBUTING.md) for details on how to get started.

Before contributing, please read our [Code of Conduct](CODE_OF_CONDUCT.md).

For maintainers: see [Releasing](docs/RELEASING.md) for the release process.

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
