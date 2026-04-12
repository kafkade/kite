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

**47 tests passing** · 5-panel dashboard · Help overlay · Settings menu

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

## Upcoming Feature Phases

See [GitHub Milestones](https://github.com/kafkade/kite/milestones) for detailed tracking:

- **Phase 2**: Hardware Sensors, GPU & Theming
- **Phase 3**: Containers, Alerts & Process Intelligence
- **Phase 4**: Remote Monitoring & Metrics Export
- **Phase 5**: Polish, Accessibility & Distribution
