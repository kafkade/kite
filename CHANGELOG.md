# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/),
and this project adheres to [Semantic Versioning](https://semver.org/).

## [Unreleased]

### Added

- Project website at `kite.kafkade.com` with brand kit, downloads, and feature overview
- Light and dark theme toggle with cookie persistence
- Interactive terminal mockup showing kite's dashboard output
- Releases section with download links for all platforms (Linux, macOS, Windows)
- Package manager section (Homebrew, Cargo, Scoop, WinGet, APT, RPM, AUR, Nix — coming soon)
- Feedback section linking to GitHub issue templates for bug reports and feature requests
- Konami code easter egg (↑↑↓↓←→←→BA) activating a Matrix-style theme with falling katakana rain
- GitHub social preview image (1280×640px) with kite branding
- SVG logo in README replacing emoji header
- FUNDING.yml enabling GitHub Sponsor button (GitHub Sponsors, Ko-fi, Buy Me a Coffee, Patreon)
- Support section on website with donation links prioritizing GitHub Sponsors and Ko-fi

### Changed

- Improved dark mode contrast — all body text now meets WCAG AA (4.5:1)
- Rebranded wordmark to lowercase `kite` to match CLI identity

## [1.0.0] - 2026-04-12

First stable release. 🎉

### Added

## [0.3.0] - 2026-04-12

### Added

- Help overlay (`?`) showing all keybindings grouped by category
- In-app settings menu (`m`) for adjusting update interval, graph symbols, and toggling panels at runtime
- Input mode system (Normal, Filtering, Help, Menu) with mode indicator in status bar
- Vim-style navigation (`j`/`k`) in process list and menus
- Force refresh keybinding (`r`)
- Status bar hint showing `? help` for discoverability

## [0.2.0] - 2026-04-12

### Added

- Interactive process table with sortable columns (PID, name, user, CPU%, memory, status, threads)
- Process tree view showing parent-child relationships with indented hierarchy
- Process search and filtering with inline filter input bar
- Process management: send signals (SIGTERM, SIGKILL, SIGSTOP, SIGCONT) and renice processes
- Confirmation dialogs for destructive actions (kill/signal) with keyboard navigation
- Process table keyboard navigation: scroll, select, sort column cycling, sort order toggle
- Process detail view showing full command line, environment, and open connections

## [0.1.0] - 2026-04-11

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

[Unreleased]: https://github.com/kafkade/kite/compare/v1.0.0...HEAD
[1.0.0]: https://github.com/kafkade/kite/compare/v0.3.0...v1.0.0
[0.3.0]: https://github.com/kafkade/kite/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/kafkade/kite/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/kafkade/kite/releases/tag/v0.1.0
