<p align="center">
  <img src="docs/website/logo.svg" alt="kite logo" width="80" height="100">
</p>

<h1 align="center">kite</h1>

[![CI](https://github.com/kafkade/kite/actions/workflows/ci.yml/badge.svg)](https://github.com/kafkade/kite/actions/workflows/ci.yml)
[![License: MIT OR Apache-2.0](https://img.shields.io/crates/l/kite)](LICENSE-MIT)
[![Rust](https://img.shields.io/badge/rust-1.85%2B-orange.svg)](https://www.rust-lang.org/)

A modern, cross-platform TUI system resource monitor written in Rust — inspired by [btop++](https://github.com/aristocratos/btop).

Kite gives you a real-time, interactive terminal dashboard for CPU, memory, disk, network, GPU, containers, and processes — with full keyboard/mouse control, customizable themes and layouts, configurable alerts, and remote monitoring over SSH.

> **Status**: ✅ Phases 1–3 complete. Core metrics, GPU/sensors, theming, containers, alerts, and advanced process features are all live.

---

## Features

### Core Dashboard

- Real-time CPU monitoring (total + per-core, frequency, load averages) with sparkline graphs
- Memory & swap usage with historical graphs and bar gauges
- Disk I/O rates and filesystem usage
- Network interface traffic with auto-scaling graphs
- Adaptive multi-panel layout (CPU, memory, disk, network, process table, and more)

### Hardware Sensors & GPU

- Temperature monitoring via Sensors panel (per-component temps, color-coded thresholds, CPU temp sparkline)
- GPU monitoring panel with NVIDIA NVML support (utilization, VRAM, temp, fan speed, clocks, power draw)
- Battery monitoring in the status bar (charge %, state, power draw) — desktops degrade gracefully

### Process Management

- Interactive process table (sort, filter, search, tree view)
- Process signals (SIGTERM, SIGKILL, SIGSTOP, SIGCONT) and renice
- Confirmation dialogs for destructive actions
- Vim-style navigation (`j`/`k`) and keyboard-driven workflow
- Per-process disk I/O stats (read/write bytes columns)
- Top-N mode: cycle All → Top 10 → Top 25 → Top 50 (`n` key)
- Process bookmarks: pin processes to the top with `b` key (`*` indicator)

### Container Monitoring

- Docker containers: name, image, status, CPU%, memory, net I/O, block I/O
- Docker actions: start, stop, restart
- Kubernetes pods: name, namespace, status, ready, restarts, age, node, resource requests/limits
- Namespace filtering for K8s
- Docker + K8s share a layout row (side-by-side when both enabled)

### Alert System

- Configurable alert rules in TOML (metric, condition, threshold, duration, severity)
- 7 metrics: CPU total, memory %, swap %, disk %, CPU temp, GPU temp, GPU utilization
- 3 severities: Info, Warning, Critical
- Status bar indicator with severity-colored alerts
- Terminal bell on critical alerts
- Alert history (last 100)
- Default rules: High CPU (>90%), Critical CPU (>95%), High Memory (>90%)

### Themes

11 built-in themes — switch with the settings menu or `--theme` CLI flag:

`default` · `dracula` · `gruvbox-dark` · `catppuccin-mocha` · `catppuccin-latte` · `nord` · `solarized-dark` · `solarized-light` · `tokyo-night` · `one-dark` · `monokai`

Custom themes can be added as TOML files in `~/.config/kite/themes/`.

### Layout Presets

6 layout presets — switch with the settings menu or `--layout` CLI flag:

`default` · `minimal` · `full` · `server` · `laptop` · `gpu-focus`

The layout engine adaptively adds or removes rows based on panel visibility.

### UI & Configuration

- Help overlay (`?`) showing all keybindings
- In-app settings menu (`m`) — adjust update interval, graph symbols, toggle panels, switch themes/layouts at runtime
- TOML-based configuration with CLI argument overrides
- Configurable update interval and keybindings
- Input mode system with status bar indicators

### Planned

- **Phase 5**: Accessibility, i18n, platform packaging

---

## Quick Start

### Install

```bash
cargo install kite-monitor
```

Or download a pre-built binary from [Releases](https://github.com/kafkade/kite/releases).

### Prerequisites

- [Rust](https://rustup.rs/) 1.85+ (edition 2024) — for building from source

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
  -i, --interval <MS>      Update interval in milliseconds [default: 1000]
      --theme <NAME>       Theme name (e.g., dracula, nord, catppuccin-mocha)
      --layout <PRESET>    Layout preset (default, minimal, full, server, laptop, gpu-focus)
      --generate-config    Generate a default config file and exit
  -h, --help               Print help
  -V, --version            Print version
```

### Keyboard Shortcuts

| Key                 | Action                              |
| ------------------- | ----------------------------------- |
| `q` / `Ctrl+C`      | Quit                                |
| `?`                 | Toggle help overlay                 |
| `m`                 | Toggle settings menu                |
| `r`                 | Force refresh                       |
| `↑`/`↓` or `j`/`k`  | Scroll process list                 |
| `←`/`→`             | Change sort column                  |
| `Space`             | Pause/unpause process updates       |
| `/`                 | Filter processes                    |
| `t`                 | Toggle tree view                    |
| `n`                 | Cycle Top-N mode (All/10/25/50)     |
| `b`                 | Toggle bookmark on selected process |
| `K`                 | Kill selected process               |
| `PgUp`/`PgDn`       | Page scroll                         |
| `Esc`               | Close overlay / clear filter        |

---

## Configuration

Kite uses a TOML config file located at:

- **Linux/macOS**: `$XDG_CONFIG_HOME/kite/config.toml` or `~/.config/kite/config.toml`
- **Windows**: `%APPDATA%\kite\config.toml`

You can also adjust settings at runtime via the settings menu (`m`).

### Feature Flags

Kite uses Cargo feature flags to control optional integrations:

| Feature      | Default | Description                               |
| ------------ | ------- | ----------------------------------------- |
| `gpu`        | ✅      | NVIDIA GPU monitoring via nvml-wrapper    |
| `battery`    | ✅      | Battery monitoring via starship-battery   |
| `docker`     | ✅      | Docker container monitoring via bollard   |
| `k8s`        | ❌      | Kubernetes pod monitoring via kube-rs     |
| `ssh`        | ❌      | SSH remote machine monitoring via russh   |
| `prometheus` | ❌      | Prometheus metrics exporter at `/metrics` |

Build with or without specific features:

```bash
cargo build --release                                   # All default features
cargo build --release --features k8s                    # Add Kubernetes support
cargo build --release --features ssh                    # Add SSH remote monitoring
cargo build --release --features prometheus             # Add Prometheus exporter
cargo build --release --features "k8s,ssh,prometheus"   # All optional features
cargo build --release --no-default-features             # Minimal build
```

### Alert Configuration

Define alert rules in your config file:

```toml
[[alerts]]
metric = "cpu_total"
condition = "above"
threshold = 90.0
duration_secs = 10
severity = "warning"

[[alerts]]
metric = "memory_percent"
condition = "above"
threshold = 95.0
duration_secs = 5
severity = "critical"
```

Supported metrics: `cpu_total`, `memory_percent`, `swap_percent`, `disk_percent`, `cpu_temp`, `gpu_temp`, `gpu_utilization`.

---

## Architecture

```
src/
├── main.rs              # Entry point, CLI args (clap), async event loop
├── app.rs               # Application state machine, input modes
├── alert/               # Alert engine and rules
│   ├── mod.rs           # AlertEngine evaluation
│   └── rules.rs         # Metric, Condition, Severity, AlertRule
├── config/              # TOML config loading, keybindings
├── collector/           # Data collectors
│   ├── cpu.rs           # CPU metrics
│   ├── memory.rs        # RAM + swap
│   ├── disk.rs          # Disk I/O + usage
│   ├── network.rs       # Network interfaces
│   ├── process.rs       # Processes (with I/O, bookmarks, top-N)
│   ├── sensor.rs        # Temperature sensors
│   ├── gpu.rs           # GPU metrics (NVIDIA)
│   ├── battery.rs       # Battery status
│   ├── docker.rs        # Docker containers
│   └── k8s.rs           # Kubernetes pods
├── ui/
│   ├── layout.rs        # Adaptive layout engine
│   ├── theme.rs         # Theme engine (11 built-in themes)
│   ├── help.rs          # Help overlay (? key)
│   ├── menu.rs          # Settings menu (m key)
│   ├── dialog.rs        # Confirmation dialogs
│   └── widgets/         # Panel widgets
│       ├── cpu_box.rs, mem_box.rs, disk_box.rs, net_box.rs
│       ├── gpu_box.rs, sensor_box.rs
│       └── proc_box.rs, container_box.rs, k8s_box.rs
├── input/               # Keyboard event handling with input modes
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

| Component   | Technology                            |
| ----------- | ------------------------------------- |
| Language    | Rust (edition 2024)                   |
| TUI         | ratatui + crossterm                   |
| Async       | tokio                                 |
| System data | sysinfo                               |
| GPU         | nvml-wrapper (optional)               |
| Battery     | starship-battery (optional)           |
| Docker      | bollard (optional)                    |
| Kubernetes  | kube-rs + k8s-openapi (optional)      |
| CLI         | clap (derive)                         |
| Config      | serde + toml                          |
| Errors      | thiserror + anyhow                    |

---

## License

Dual-licensed under [MIT](LICENSE-MIT) or [Apache 2.0](LICENSE-APACHE) at your option.
