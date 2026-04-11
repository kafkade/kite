# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/),
and this project adheres to [Semantic Versioning](https://semver.org/).

## [Unreleased]

### Added
- Phase 1 Stage 6: Help overlay, settings menu, vim keys, responsive layout, color detection (planned)

## [0.1.0] - 2025-04-11

### Added
- Real-time CPU monitoring (total + per-core usage, frequency, load averages) with braille line graphs
- Memory and swap usage monitoring with bar gauges and history graphs
- Disk I/O rates and filesystem usage with partition bars and I/O graphs
- Network interface traffic monitoring with auto-scaling bandwidth graphs
- Interactive process table with sorting, filtering, search, tree view
- Process management: kill, signal (SIGTERM, SIGKILL, SIGSTOP, SIGCONT), renice
- Confirmation dialogs for destructive process actions
- TOML-based configuration system with CLI argument overrides
- Configurable update interval via `--interval` flag
- Customizable keybindings
- RAII terminal guard for clean exit on panic
- Event-driven async architecture with tokio
- 4-panel grid layout (CPU, memory, disk, network) + process table
- Cross-platform support (Linux, macOS, Windows)

[Unreleased]: https://github.com/kafkade/kite/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/kafkade/kite/releases/tag/v0.1.0
