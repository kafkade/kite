# Kite — Implementation Plan

> A cross-platform TUI system resource monitor written in Rust.
> Full spec: [docs/SPEC.md](docs/SPEC.md)

---

## Phase 1: Foundation & Core Metrics (MVP)

**Goal**: A working TUI that displays real-time CPU, memory, disk, network stats with a process list.

| # | Stage | Status | Description |
|---|-------|--------|-------------|
| 1 | Skeleton & Event Loop | ✅ Done | Cargo init, tokio event loop, ratatui terminal setup/teardown, panic hook, quit handling |
| 2 | Configuration System | ✅ Done | TOML config, CLI args (clap), Collector trait, configurable tick interval |
| 3 | CPU & Memory | ✅ Done | CPU + memory collectors via sysinfo, braille graphs, per-core bars, bar gauges |
| 4 | Disk & Network | ✅ Done | Disk + network collectors, I/O graphs, bandwidth graphs, 4-panel layout |
| 5 | Process Table | ✅ Done | Sortable process table, tree view, search/filter, kill/signal/renice |
| 6 | Polish & Help | ⬜ Not Started | Help overlay, settings menu, vim keys, responsive layout, color detection |

### Stage Details

#### Stage 1: Skeleton & Event Loop
- `cargo init` with all Phase 1 dependencies
- `main.rs` → CLI entry point
- `app.rs` → Application state machine (Running, Quitting)
- `util/error.rs` → Error types (thiserror)
- `util/ring_buffer.rs` → Generic fixed-size ring buffer for graph history
- `util/units.rs` → Human-readable formatting (bytes, %, durations)
- `ui/mod.rs` + `ui/layout.rs` → ratatui alternate screen, panic hook
- `input/mod.rs` + `input/keyboard.rs` → crossterm event reader
- Event loop: `tokio::select!` multiplexing input + tick timer
- Status bar with hostname, clock, uptime placeholder
- **Exit criteria**: blank TUI frame, quits cleanly on `q` / `Ctrl+C`

#### Stage 2: Configuration System
- `config/mod.rs` → Config loader (XDG_CONFIG_HOME / fallback paths)
- `config/settings.rs` → Typed `Config` struct with serde defaults
- `config/keybindings.rs` → Keybinding map (action → key)
- Priority: CLI args > env vars > config file > defaults
- `collector/mod.rs` → `Collector` trait + `CollectorScheduler`
- `kite --generate-config` to emit default config
- **Exit criteria**: app loads config, `--interval 500` changes tick rate

#### Stage 3: CPU & Memory Collectors + Widgets
- `collector/cpu.rs` → total %, per-core %, frequency, model, load avg, uptime
- `collector/memory.rs` → RAM (total/used/free/available/cached), swap
- `ui/widgets/cpu_box.rs` → braille line graph, per-core bars, text stats
- `ui/widgets/mem_box.rs` → bar gauges, history graph, text breakdown
- Ring buffer history for both collectors
- **Exit criteria**: 2 live panels updating every tick

#### Stage 4: Disk & Network Collectors + Widgets
- `collector/disk.rs` → mounts, fs type, space, I/O rates per disk
- `collector/network.rs` → interfaces, IPs, upload/download speeds, totals
- `ui/widgets/disk_box.rs` → partition bars, I/O graph, mount list
- `ui/widgets/net_box.rs` → auto-scaling bandwidth graph, interface list
- 4-panel grid layout, basic mouse scroll support
- **Exit criteria**: 4 live panels — CPU, memory, disk, network

#### Stage 5: Process Table & Management
- `collector/process.rs` → PID, name, user, CPU%, MEM%, state, threads, tree
- `ui/widgets/proc_box.rs` → sortable table, tree view, search bar, scroll
- Process actions: SIGTERM, SIGKILL, SIGSTOP, SIGCONT, custom signal, renice
- `ui/dialog.rs` → confirmation dialogs for destructive actions
- Detail panel (Enter): command line, env vars, open files, connections
- **Exit criteria**: full process manager with kill/filter/sort/tree

#### Stage 6: Polish, Help & Input Refinement
- `ui/help.rs` → contextual help overlay (`?`)
- `ui/menu.rs` → in-app settings menu (`m`)
- Input modes: Normal / Search (`/`) / Menu / Help
- Vim-style navigation preset (hjkl)
- Status bar: hostname, clock, uptime, interval, active panel
- Responsive layout (min 80×24, graceful degradation)
- Color auto-detection (truecolor > 256 > 16) with `--color-mode`
- Terminal resize handling
- **Exit criteria**: complete, polished Phase 1 MVP

---

## Phase 2: Hardware Sensors, GPU & Theming ⬜

*Details in [docs/SPEC.md](docs/SPEC.md) §4 Phase 2*

## Phase 3: Containers, Alerts & Process Intelligence ⬜

*Details in [docs/SPEC.md](docs/SPEC.md) §4 Phase 3*

## Phase 4: Remote Monitoring & Metrics Export ⬜

*Details in [docs/SPEC.md](docs/SPEC.md) §4 Phase 4*

## Phase 5: Polish, Accessibility & Distribution ⬜

*Details in [docs/SPEC.md](docs/SPEC.md) §4 Phase 5*
