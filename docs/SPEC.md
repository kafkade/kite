# Project Prompt: **Kite** — A Modern Cross-Platform System Resource Monitor

## 1. Vision & Overview

Build **Kite**, a modern, feature-rich, cross-platform terminal UI (TUI) system resource monitor inspired by [btop++](https://github.com/aristocratos/btop). Kite goes beyond btop by adding GPU monitoring, container orchestration visibility, remote machine monitoring via SSH, configurable alerts, and metrics export — all within a beautiful, highly customizable terminal interface.

**Name rationale:** "Kite" evokes soaring above your system, seeing everything below with a bird's-eye view. It's short (4 chars), easy to type in a terminal, memorable, and the crate name is available on crates.io. Unrelated to the htop/btop naming lineage.

---

## 2. Tech Stack

| Layer | Technology | Rationale |
|---|---|---|
| **Language** | **Rust** (edition 2024) | Memory safety without GC, fearless concurrency, excellent cross-platform support, minimal runtime overhead — ideal for a performance-critical system monitor |
| **TUI Framework** | **ratatui** + **crossterm** backend | The de facto Rust TUI framework. ratatui provides widgets (charts, tables, gauges, sparklines); crossterm provides cross-platform terminal manipulation |
| **Async Runtime** | **tokio** | For concurrent data collection, SSH connections, and non-blocking I/O |
| **System Metrics** | **sysinfo** crate | Cross-platform CPU, memory, disk, network, process data |
| **GPU Metrics** | **nvml-wrapper** (NVIDIA), platform-specific APIs for AMD/Intel | GPU utilization, VRAM, temperature, clock speeds |
| **Hardware Sensors** | **lm-sensors** bindings (Linux), **IOKit** (macOS), **WMI/OpenHardwareMonitor** (Windows) | Temperature, fan speed, voltage readings |
| **Container Monitoring** | **bollard** (Docker API), **kube-rs** (Kubernetes API) | Native async Rust clients for Docker and K8s |
| **SSH / Remote** | **russh** or **openssh** crate | Async SSH2 client for remote machine monitoring |
| **Configuration** | **TOML** via **toml** crate | Human-readable config files, consistent with Rust ecosystem conventions |
| **Metrics Export** | **prometheus-client** crate | Expose `/metrics` endpoint for Prometheus scraping |
| **Logging / File Export** | **tracing** + **tracing-subscriber** | Structured logging framework; file appender for metric logs |
| **Serialization** | **serde** + **serde_json** | For config, theme, and data serialization |
| **Build / CI** | **cargo** + **cross** (cross-compilation) + **GitHub Actions** | Reproducible builds for Linux, macOS, Windows; ARM64 support |
| **Testing** | **cargo test** + **insta** (snapshot testing) + **criterion** (benchmarks) | Unit, integration, snapshot, and performance tests |
| **Packaging** | **cargo-deb**, **cargo-rpm**, **homebrew formula**, **winget manifest**, **snap** | Native package distribution for all platforms |

---

## 3. Architecture

### 3.1 High-Level Architecture

```
┌──────────────────────────────────────────────────────────────┐
│                        TUI Layer (ratatui)                   │
│  ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌───────┐ │
│  │ CPU Box │ │ Mem Box │ │ Net Box │ │ Disk Box│ │GPU Box│ │
│  └────┬────┘ └────┬────┘ └────┬────┘ └────┬────┘ └───┬───┘ │
│       │            │           │            │          │      │
│  ┌────┴────────────┴───────────┴────────────┴──────────┴───┐ │
│  │                    Layout Engine                         │ │
│  │         (dynamic grid, user-configurable panels)        │ │
│  └──────────────────────┬──────────────────────────────────┘ │
│                         │                                    │
│  ┌──────────────────────┴──────────────────────────────────┐ │
│  │                   Event Loop (tokio)                     │ │
│  │        Input handling ◄──► Data refresh ◄──► Render      │ │
│  └──────────────────────┬──────────────────────────────────┘ │
└─────────────────────────┼────────────────────────────────────┘
                          │
┌─────────────────────────┼────────────────────────────────────┐
│                   Data Collection Layer                       │
│  ┌──────┐ ┌──────┐ ┌──────┐ ┌──────┐ ┌──────┐ ┌──────────┐ │
│  │ CPU  │ │ Mem  │ │ Disk │ │ Net  │ │ GPU  │ │ Sensors  │ │
│  │Collect│ │Collect│ │Collect│ │Collect│ │Collect│ │ Collect  │ │
│  └──┬───┘ └──┬───┘ └──┬───┘ └──┬───┘ └──┬───┘ └────┬─────┘ │
│     │        │        │        │        │           │        │
│  ┌──┴────────┴────────┴────────┴────────┴───────────┴──────┐ │
│  │              Platform Abstraction Layer (PAL)            │ │
│  │     Linux impl  │  macOS impl  │  Windows impl          │ │
│  └─────────────────────────────────────────────────────────┘ │
└──────────────────────────────────────────────────────────────┘
                          │
┌─────────────────────────┼────────────────────────────────────┐
│                   Integration Layer                           │
│  ┌───────────┐  ┌────────────┐  ┌────────────┐  ┌─────────┐ │
│  │  Docker   │  │ Kubernetes │  │    SSH      │  │Prometheus│ │
│  │  Client   │  │   Client   │  │  Remote     │  │ Exporter │ │
│  └───────────┘  └────────────┘  └────────────┘  └─────────┘ │
└──────────────────────────────────────────────────────────────┘
```

### 3.2 Module Structure

```
kite/
├── Cargo.toml
├── src/
│   ├── main.rs                  # Entry point, CLI arg parsing (clap)
│   ├── app.rs                   # Application state machine
│   ├── config/
│   │   ├── mod.rs               # Config loading, merging (CLI > env > file > defaults)
│   │   ├── settings.rs          # Typed config struct
│   │   └── keybindings.rs       # Keybinding map
│   ├── collector/
│   │   ├── mod.rs               # Collector trait + scheduler
│   │   ├── cpu.rs               # CPU metrics
│   │   ├── memory.rs            # RAM + swap metrics
│   │   ├── disk.rs              # Disk I/O + usage
│   │   ├── network.rs           # Network interfaces + traffic
│   │   ├── process.rs           # Process list + tree
│   │   ├── gpu.rs               # GPU metrics (NVIDIA/AMD/Intel)
│   │   ├── sensors.rs           # Temperature, fan, voltage
│   │   ├── battery.rs           # Battery status
│   │   └── platform/
│   │       ├── linux.rs
│   │       ├── macos.rs
│   │       └── windows.rs
│   ├── ui/
│   │   ├── mod.rs               # Root render function
│   │   ├── layout.rs            # Dynamic layout engine
│   │   ├── theme.rs             # Theme parser + applicator
│   │   ├── widgets/
│   │   │   ├── cpu_box.rs       # CPU panel (graph + per-core bars)
│   │   │   ├── mem_box.rs       # Memory panel (usage + swap)
│   │   │   ├── net_box.rs       # Network panel (speed graph)
│   │   │   ├── disk_box.rs      # Disk panel (I/O + mount info)
│   │   │   ├── gpu_box.rs       # GPU panel
│   │   │   ├── proc_box.rs      # Process table + tree view
│   │   │   ├── sensor_box.rs    # Sensors panel
│   │   │   ├── battery_box.rs   # Battery meter
│   │   │   └── container_box.rs # Docker/K8s panel
│   │   ├── menu.rs              # In-app menu system
│   │   ├── help.rs              # Help overlay
│   │   └── dialog.rs            # Confirmation dialogs
│   ├── input/
│   │   ├── mod.rs               # Input event dispatcher
│   │   ├── keyboard.rs          # Key event handling
│   │   └── mouse.rs             # Mouse event handling
│   ├── alert/
│   │   ├── mod.rs               # Alert engine
│   │   ├── rules.rs             # Threshold rules
│   │   └── notify.rs            # Notification dispatch (terminal bell, desktop notif)
│   ├── export/
│   │   ├── mod.rs               # Export coordinator
│   │   ├── prometheus.rs        # Prometheus /metrics endpoint
│   │   ├── json_log.rs          # JSON line log exporter
│   │   └── csv_log.rs           # CSV log exporter
│   ├── remote/
│   │   ├── mod.rs               # Remote monitoring coordinator
│   │   ├── ssh.rs               # SSH connection manager
│   │   └── agent.rs             # Lightweight remote agent protocol
│   ├── container/
│   │   ├── mod.rs               # Container abstraction
│   │   ├── docker.rs            # Docker API client
│   │   └── kubernetes.rs        # K8s API client
│   └── util/
│       ├── mod.rs
│       ├── ring_buffer.rs       # Fixed-size history buffer for graphs
│       ├── units.rs             # Human-readable unit formatting
│       └── error.rs             # Error types
├── themes/
│   ├── default.toml
│   ├── dracula.toml
│   ├── gruvbox.toml
│   ├── catppuccin.toml
│   ├── nord.toml
│   ├── solarized-dark.toml
│   └── tokyo-night.toml
├── config/
│   └── default.toml             # Default configuration
├── tests/
│   ├── integration/
│   └── snapshots/
├── benches/
│   └── collector_bench.rs
└── docs/
    ├── ARCHITECTURE.md
    ├── CONFIGURATION.md
    ├── THEMES.md
    └── KEYBINDINGS.md
```

### 3.3 Core Design Patterns

1. **Collector Trait Pattern**: Each data source implements a `Collector` trait with `async fn collect(&mut self) -> Result<MetricSnapshot>`. Collectors are registered in a scheduler that polls them at configurable intervals.

2. **Ring Buffer History**: All time-series data (CPU %, network throughput, etc.) is stored in fixed-size ring buffers for graph rendering. Configurable history depth (default: 300 data points).

3. **Event-Driven Architecture**: The main loop uses a `tokio::select!` to multiplex:
   - Terminal input events (keyboard/mouse via crossterm)
   - Data collection ticks (configurable interval, default 1s)
   - Alert evaluations
   - Remote data arrival

4. **Platform Abstraction Layer (PAL)**: Compile-time `#[cfg(target_os)]` gates select platform-specific implementations. Each platform module implements the same trait interface.

5. **Layout Engine**: A constraint-based layout system that maps named "boxes" to terminal regions. Users define layouts in config via a simple DSL or preset names (e.g., `"default"`, `"minimal"`, `"full"`).

---

## 4. Feature Phases

### Phase 1: Foundation & Core Metrics *(MVP)*

**Goal**: A working TUI that displays real-time CPU, memory, disk, and network stats with a process list. Ship a usable tool.

#### Features

**1.1 — Project Scaffolding**
- Initialize Rust workspace with `cargo init`
- Set up CI/CD with GitHub Actions (lint, test, build for Linux/macOS/Windows)
- Configure `clippy`, `rustfmt`, and `cargo-deny` for code quality
- Set up cross-compilation with `cross` for ARM64 targets
- Create README, LICENSE (MIT + Apache 2.0 dual), CONTRIBUTING.md

**1.2 — Configuration System**
- TOML config file at `$XDG_CONFIG_HOME/kite/config.toml` (or `~/.config/kite/config.toml`)
- CLI argument parsing with `clap` (override any config value)
- Config priority: CLI args > environment variables > config file > defaults
- Configurable update interval (default: 1000ms, min: 100ms)
- Configurable graph symbol set (braille, block, tty-safe)
- In-app settings menu to modify configuration live (changes saved to config file)

**1.3 — CPU Monitoring**
- Total CPU usage percentage with historical line graph (braille characters)
- Per-core usage with horizontal bar indicators
- CPU frequency (current, min, max) per core
- CPU model name, architecture, core/thread count
- Load averages (1m, 5m, 15m) on Linux/macOS
- Uptime display
- Graph auto-scaling and configurable history depth

**1.4 — Memory Monitoring**
- Total, used, free, available, cached, buffered RAM
- Swap usage (total, used, free)
- Memory usage historical graph
- Visual bar gauges with percentage labels
- Per-process memory breakdown (RSS, VMS, shared)

**1.5 — Disk Monitoring**
- List all mounted filesystems with mount point, filesystem type, total/used/free space
- Disk I/O rates (read/write bytes/sec, IOPS) per physical disk
- I/O activity historical graph
- Visual usage bars per partition
- Filter/hide specific mount points via config

**1.6 — Network Monitoring**
- List all network interfaces with IP addresses
- Upload/download speed per interface (bytes/sec)
- Total transferred (cumulative session and all-time)
- Auto-scaling bandwidth graph
- Interface selection (show specific or all)

**1.7 — Process Management**
- Sortable process table (PID, name, user, CPU%, MEM%, state, threads, command)
- Process tree view (parent-child hierarchy with collapsible nodes)
- Process filtering/search (by name, PID, user, regex)
- Sort by any column (ascending/descending toggle)
- Send signals to processes: SIGTERM, SIGKILL, SIGSTOP, SIGCONT, and any signal by number
- Renice (change priority) of selected process
- Detailed process info panel (open files, network connections, environment vars, command line)
- Scroll through process list with keyboard and mouse
- Pause/unpause process list refresh

**1.8 — TUI Rendering Engine**
- Main layout with CPU, memory, network, disk, and process boxes
- Responsive layout: adapt to terminal size (min 80×24, graceful degradation)
- 256-color and truecolor support with fallback to 16 colors
- Unicode braille/block character graphs
- Full mouse support: clickable buttons, scroll in lists, drag to resize panels
- Smooth animation: interpolated graph updates, fade transitions
- Status bar with clock, hostname, uptime, update interval

**1.9 — Input System**
- Keyboard navigation across all panels
- Configurable keybindings (loaded from config TOML)
- Vim-style navigation (hjkl) as optional preset
- Modal input: normal mode, search mode, menu mode
- Global shortcuts: quit (q), help (?), menu (m), refresh (r)

---

### Phase 2: Hardware Sensors, GPU & Theming

**Goal**: Add hardware depth (temperatures, GPU, battery) and full visual customization.

#### Features

**2.1 — Temperature & Hardware Sensors**
- CPU temperature (per-core and package) via:
  - Linux: `/sys/class/hwmon` or `lm-sensors`
  - macOS: `IOKit` SMC interface
  - Windows: WMI or OpenHardwareMonitor shared memory
- Disk drive temperatures (SMART data where available)
- Fan speed readings (RPM)
- Voltage sensors
- Temperature warning thresholds (configurable, visual indicator when exceeded)
- Historical temperature graph

**2.2 — GPU Monitoring**
- **NVIDIA GPUs** via NVML:
  - GPU utilization %, memory utilization %
  - VRAM usage (total, used, free)
  - GPU temperature, fan speed
  - Clock speeds (core, memory)
  - Power draw (current, limit)
  - Running GPU processes with VRAM usage
- **AMD GPUs** via ROCm SMI / sysfs (Linux):
  - Utilization, VRAM, temperature, clocks, power
- **Intel GPUs** via `intel_gpu_top` / sysfs (Linux):
  - Utilization, clock speed
- **Apple Silicon GPU** (macOS):
  - Utilization via IOKit
- Multi-GPU support: toggle visibility per GPU (keys 5,6,7,8)
- GPU info in process detail view (which processes use GPU)
- Dedicated GPU box with utilization graph and VRAM bar

**2.3 — Battery Monitoring**
- Battery percentage, charge status (charging, discharging, full, not present)
- Estimated time remaining
- Battery health (design capacity vs current capacity)
- Power draw (watts)
- Compact battery meter in status bar

**2.4 — Theming Engine**
- TOML-based theme files with named color tokens:
  ```toml
  [colors]
  background = "#1a1b26"
  foreground = "#c0caf5"
  accent = "#7aa2f7"
  cpu_graph = "#f7768e"
  mem_graph = "#9ece6a"
  net_upload = "#e0af68"
  net_download = "#7dcfff"
  alert_warning = "#e0af68"
  alert_critical = "#f7768e"
  ```
- Ship 10+ built-in themes: Default, Dracula, Gruvbox, Catppuccin (Mocha/Latte), Nord, Solarized Dark/Light, Tokyo Night, One Dark, Monokai
- Theme hot-reload: change theme file → auto-apply without restart
- Theme preview in settings menu
- Support for 256-color, truecolor, and 16-color fallback modes
- Custom graph symbol sets per theme
- Transparent background support (inherit terminal background)

**2.5 — Custom Layouts & Presets**
- Layout presets: `default`, `minimal`, `full`, `server`, `laptop`, `gpu-focus`
- Custom layout definition via config:
  ```toml
  [layout]
  preset = "custom"
  rows = [
    ["cpu:60%", "gpu:40%"],
    ["memory:30%", "network:30%", "disk:40%"],
    ["processes:100%"]
  ]
  ```
- Toggle individual boxes on/off with keyboard shortcuts
- Save current layout as named preset
- Panel resize via mouse drag on borders

---

### Phase 3: Containers, Alerts & Process Intelligence

**Goal**: Add container awareness, smart alerting, and advanced process features.

#### Features

**3.1 — Docker Container Monitoring**
- List all containers (running, stopped, paused) with:
  - Container name, image, status, uptime
  - CPU usage %, memory usage (and limit)
  - Network I/O (rx/tx bytes)
  - Block I/O (read/write)
  - PIDs inside the container
- Container actions: start, stop, restart, pause, unpause, kill
- Container log viewer (tail last N lines)
- Docker Compose service grouping
- Auto-detect Docker socket (`/var/run/docker.sock` or TCP)
- Container detail panel with environment, ports, volumes, mounts

**3.2 — Kubernetes Pod Monitoring**
- Connect to current kubeconfig context or specified context
- List pods with: name, namespace, status, restarts, age, node
- Pod resource usage: CPU, memory (requests vs limits vs actual)
- Container-level metrics within pods
- Pod actions: delete, describe, logs
- Namespace filtering
- Node overview (CPU/memory usage across cluster nodes)
- K8s events stream

**3.3 — Alert System**
- Configurable alert rules in TOML:
  ```toml
  [[alerts]]
  name = "High CPU"
  metric = "cpu.total"
  condition = "> 90"
  duration = "30s"
  severity = "warning"
  
  [[alerts]]
  name = "Disk Full"
  metric = "disk./.usage_percent"
  condition = "> 95"
  severity = "critical"
  ```
- Alert severities: `info`, `warning`, `critical`
- Alert actions:
  - Terminal bell
  - Visual indicator (flashing/color change in affected panel)
  - Desktop notification (via `notify-rust` crate)
  - Execute custom command
  - Log to alert history
- Alert history view (last N triggered alerts with timestamps)
- Alert suppression / snooze (mute an alert for X minutes)
- Hysteresis: require metric to stay below threshold for N seconds before clearing

**3.4 — Advanced Process Features**
- Per-process I/O stats (read/write bytes, syscalls)
- Per-process network connections (socket list, remote address, state)
- cgroup info (which cgroup, limits, usage) — Linux
- Process environment variables viewer
- Open files / file descriptors list
- Process timeline (CPU/mem history for selected process)
- "Top N" mode: only show top consumers by CPU, memory, I/O, or network
- Process bookmarks: pin processes to always show at top

---

### Phase 4: Remote Monitoring & Metrics Export

**Goal**: Monitor remote machines and export data for external consumption.

#### Features

**4.1 — SSH Remote Monitoring**
- SSH connection manager (stored in config, supports key auth + agent forwarding):
  ```toml
  [[remotes]]
  name = "prod-web-1"
  host = "10.0.1.5"
  port = 22
  user = "monitor"
  key = "~/.ssh/id_ed25519"
  ```
- Deploy lightweight data-collection agent over SSH (embedded binary or script)
- Multi-machine view: tab-switch between machines or split-screen
- Connection health indicator (latency, reconnect on failure)
- Aggregate view: combined graphs across all machines
- Secure: no credentials stored in plaintext (use SSH agent, keychain integration)

**4.2 — Prometheus Metrics Export**
- Embedded HTTP server (optional, off by default) on configurable port
- Expose all collected metrics in Prometheus text format at `/metrics`
- Metric naming convention: `kite_cpu_usage_percent`, `kite_memory_used_bytes`, etc.
- Labels for per-core, per-disk, per-interface, per-gpu breakdown
- Custom metric labels via config
- Basic auth or token auth for the metrics endpoint
- Grafana dashboard template (JSON) shipped with the project

**4.3 — File-Based Metrics Logging**
- Log metrics to file in JSON Lines or CSV format
- Configurable log rotation (by size or time)
- Configurable metrics to log (select which collectors)
- Compression of rotated logs (gzip)
- Log directory configurable (default: `$XDG_DATA_HOME/kite/logs/`)

**4.4 — Data Replay Mode**
- Load a metrics log file and replay it in the TUI
- Seek forward/backward through time
- Useful for post-mortem analysis of incidents

---

### Phase 5: Polish, Accessibility & Distribution

**Goal**: Production-ready quality, accessibility, and wide distribution.

#### Features

**5.1 — Accessibility**
- Screen reader compatible mode (structured text output, no animations)
- High-contrast theme
- Colorblind-safe theme (deuteranopia, protanopia, tritanopia friendly palettes)
- Configurable minimum contrast ratio
- Reduced-motion mode (disable animations/transitions)

**5.2 — Performance Optimization**
- Benchmark suite with `criterion`
- Target: <1% CPU usage at default refresh rate on modern hardware
- Efficient diff-based rendering (only redraw changed cells)
- Lazy collection: only collect data for visible panels
- Memory budget: <30MB RSS for typical usage
- Start-up time: <200ms to first render

**5.3 — Internationalization (i18n)**
- Externalized strings via `fluent-rs` or similar
- Ship with English; community translations welcome
- RTL layout support (future)
- Locale-aware number/date formatting

**5.4 — Packaging & Distribution**
- **Linux**: `.deb` (Debian/Ubuntu), `.rpm` (Fedora/RHEL), AUR (Arch), Snap, Flatpak, AppImage, Nix
- **macOS**: Homebrew formula, MacPorts
- **Windows**: winget manifest, Scoop bucket, Chocolatey package, standalone `.exe` / `.msi`
- **Cross-platform**: `cargo install kite-monitor` (crates.io)
- Static binaries (musl on Linux) for zero-dependency deployment
- Auto-update check (opt-in, checks GitHub releases)

**5.5 — Documentation & Community**
- Comprehensive man page (`kite.1`)
- Built-in help system (? key shows contextual help)
- Online docs site (mdBook or similar)
- CONFIGURATION.md with every option documented
- THEMES.md with theme creation guide
- CONTRIBUTING.md with architecture overview for contributors
- Example configs for common use cases (server, laptop, developer workstation, GPU ML rig)

**5.6 — Plugin / Extension System** *(Stretch Goal)*
- Lua or WASM-based plugin API for custom collectors
- Plugins can add new panels, metrics, alert actions
- Plugin repository / registry
- Example plugins: Postgres stats, Redis info, custom application metrics

---

## 5. Non-Functional Requirements

| Requirement | Target |
|---|---|
| **Startup time** | < 200ms to first render |
| **CPU overhead** | < 1% at 1s refresh on modern hardware |
| **Memory usage** | < 30MB RSS typical, < 50MB with all features |
| **Min terminal** | 80×24 characters (graceful degradation) |
| **Refresh rate** | Configurable 100ms–10s (default 1s) |
| **Color support** | Truecolor > 256-color > 16-color (auto-detect with override) |
| **Binary size** | < 15MB stripped (static Linux musl build) |
| **Crash recovery** | Graceful terminal restore on panic (reset cursor, colors, alt screen) |
| **Security** | No root required for basic monitoring; elevated for process signals / sensor access |

---

## 6. Key Design Decisions & Constraints

1. **No runtime dependencies**: Ship a single static binary. All optional features (GPU, Docker, K8s, SSH) gracefully degrade if the underlying system APIs are unavailable.

2. **Graceful degradation**: If a sensor/API is unavailable, show "N/A" instead of crashing. Feature flags at compile time for optional heavy dependencies.

3. **Config-driven**: Every visual and behavioral aspect should be configurable via the TOML config. The app should be usable out-of-the-box with zero configuration.

4. **Incremental adoption**: Each phase produces a standalone, shippable product. Phase 1 alone is a complete resource monitor.

5. **Testing strategy**:
   - Unit tests for all collectors (mock system data)
   - Snapshot tests for UI rendering (via `insta`)
   - Integration tests with real system data (CI matrix: Linux, macOS, Windows)
   - Benchmark tests for performance regression detection
   - Fuzz testing for config/theme file parsing

6. **Terminal compatibility**: Test against: kitty, alacritty, wezterm, iTerm2, Terminal.app, Windows Terminal, GNOME Terminal, tmux, screen.

---

## 7. Implementation Guidelines

- Follow Rust 2024 edition idioms; use `clippy::pedantic` lint level
- Use `thiserror` for library errors, `anyhow` in binary/main
- Prefer `tokio::sync::watch` channels for collector → UI data flow
- Use `Arc<RwLock<>>` sparingly; prefer message passing
- Document all public APIs with `///` doc comments
- Each collector should be independently testable with mock data
- Use feature flags for optional integrations: `gpu`, `docker`, `kubernetes`, `ssh`, `prometheus`
- Default features: everything except `ssh` and `kubernetes`
- Write a `ARCHITECTURE.md` before starting Phase 1 code

---

## 8. Success Criteria

By the end of all phases, Kite should:
- ✅ Be installable on Linux, macOS, and Windows with a single binary
- ✅ Display real-time CPU, memory, disk, network, GPU, sensor, and battery metrics
- ✅ Manage processes (kill, renice, signal, filter, tree view)
- ✅ Monitor Docker containers and Kubernetes pods
- ✅ Connect to remote machines via SSH for monitoring
- ✅ Export metrics to Prometheus and log files
- ✅ Alert on configurable thresholds
- ✅ Be highly customizable (themes, layouts, keybindings)
- ✅ Use <1% CPU and <30MB RAM during normal operation
- ✅ Start in under 200ms
- ✅ Recover gracefully from any error without corrupting the terminal
