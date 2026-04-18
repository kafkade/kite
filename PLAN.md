# Kite — Implementation Plan

> A cross-platform TUI system resource monitor written in Rust.
> Full spec: [docs/SPEC.md](docs/SPEC.md) · Issues & milestones: [GitHub Issues](https://github.com/kafkade/kite/issues)

---

## Phase 1: Foundation & Core Metrics (MVP) ✅

| # | Stage | Status |
|---|-------|--------|
| 1 | Skeleton & Event Loop | ✅ Done |
| 2 | Configuration System | ✅ Done |
| 3 | CPU & Memory | ✅ Done |
| 4 | Disk & Network | ✅ Done |
| 5 | Process Table | ✅ Done |
| 6 | Polish & Help | ✅ Done |

**47 tests** (102+ total across all phases) · 5-panel dashboard · Help overlay · Settings menu

## OSS Infrastructure ✅

| Task | Status |
|------|--------|
| LICENSE files (MIT + Apache-2.0) | ✅ Done |
| CONTRIBUTING.md, CODE_OF_CONDUCT.md, SECURITY.md, SUPPORT.md | ✅ Done |
| CI workflow (test, clippy, fmt) | ✅ Done |
| Release workflow (cross-platform binaries on tag push) | ✅ Done |
| GitHub rulesets (via github-infra IaC) | ✅ Done |
| Issue templates, PR template, CODEOWNERS | ✅ Done |
| Dependabot (Cargo + Actions) | ✅ Done |
| CHANGELOG.md (Keep a Changelog) | ✅ Done |
| README badges, Cargo.toml metadata | ✅ Done |

## Remaining OSS Tasks

All tracked as GitHub Issues with full context:

- [#25](https://github.com/kafkade/kite/issues/25) — CI matrix + MSRV
- [#32](https://github.com/kafkade/kite/issues/32) — Cargo-deny
- [#27](https://github.com/kafkade/kite/issues/27) — Discussions
- [#28](https://github.com/kafkade/kite/issues/28) — Funding
- [#29](https://github.com/kafkade/kite/issues/29) — Website
- [#30](https://github.com/kafkade/kite/issues/30) — OpenSSF badge
- [#33](https://github.com/kafkade/kite/issues/33) — Contributor recognition
- [#34](https://github.com/kafkade/kite/issues/34) — Governance doc

## Phase 2: Hardware Sensors, GPU & Theming ✅

| # | Feature | Issue | Status |
|---|---------|-------|--------|
| 1 | Temperature & hardware sensors | #6 | ✅ Done |
| 2 | GPU monitoring (NVIDIA) | #7 | ✅ Done |
| 3 | Battery monitoring | #8 | ✅ Done |
| 4 | Theming engine (11 themes) | #9 | ✅ Done |
| 5 | Custom layouts & presets | #10 | ✅ Done |

**102+ tests passing** · Sensors + GPU + Battery panels · 11 built-in themes · 6 layout presets · `--theme` and `--layout` CLI flags

## Phase 3: Containers, Alerts & Process Intelligence ✅

| # | Feature | Issue | Status |
|---|---------|-------|--------|
| 1 | Docker container monitoring | #11 | ✅ Done |
| 2 | Kubernetes pod monitoring | #12 | ✅ Done |
| 3 | Alert system | #13 | ✅ Done |
| 4 | Advanced process features | #14 | ✅ Done |

**102+ tests passing** · Docker + K8s panels · Configurable alerts · Per-process I/O · Top-N mode · Process bookmarks

## Upcoming Feature Phases

See [GitHub Milestones](https://github.com/kafkade/kite/milestones) for detailed tracking:

- **Phase 4**: Remote Monitoring & Metrics Export
- **Phase 5**: Polish, Accessibility & Distribution
