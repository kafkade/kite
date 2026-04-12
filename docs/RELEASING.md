# Releasing Kite

This document describes how to maintain the changelog, create releases, and how the automated release workflow works.

---

## Changelog Maintenance

Kite uses [Keep a Changelog](https://keepachangelog.com/) format with [Semantic Versioning](https://semver.org/).

### How to update the changelog

**Every PR that changes user-facing behavior should update `CHANGELOG.md`.**

1. Open `CHANGELOG.md`
2. Add your entry under the `## [Unreleased]` section
3. Use the appropriate category:

| Category | When to use |
|----------|-------------|
| `Added` | New features |
| `Changed` | Changes to existing functionality |
| `Deprecated` | Features that will be removed in a future release |
| `Removed` | Features that were removed |
| `Fixed` | Bug fixes |
| `Security` | Security vulnerability fixes |

Example:

```markdown
## [Unreleased]

### Added
- GPU temperature monitoring for NVIDIA cards

### Fixed
- CPU percentage exceeding 100% on systems with frequency scaling
```

### Rules

- **Always write entries from the user's perspective** — "Fixed crash when resizing terminal", not "Fixed null pointer in layout.rs"
- **One entry per logical change**, not per commit
- **Link to issues/PRs** when relevant — `Fixed crash on resize (#42)`
- **Don't add entries for** refactoring, CI changes, or internal-only changes

---

## Release Process

### Step 1 — Finalize the changelog

Move entries from `[Unreleased]` to a new version section:

```markdown
## [Unreleased]

## [0.2.0] - 2026-05-15

### Added
- GPU temperature monitoring for NVIDIA cards

### Fixed
- CPU percentage exceeding 100% on systems with frequency scaling
```

Update the comparison links at the bottom of the file:

```markdown
[Unreleased]: https://github.com/kafkade/kite/compare/v0.2.0...HEAD
[0.2.0]: https://github.com/kafkade/kite/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/kafkade/kite/releases/tag/v0.1.0
```

### Step 2 — Bump the version in Cargo.toml

```toml
[package]
version = "0.2.0"
```

### Step 3 — Commit, tag, and push

```bash
git add CHANGELOG.md Cargo.toml Cargo.lock
git commit -m "chore: release v0.2.0"
git tag -a v0.2.0 -m "v0.2.0"
git push origin main --follow-tags
```

### Step 4 — The release workflow takes over

Once you push the `v*` tag, the GitHub Actions release workflow (`.github/workflows/release.yml`) automatically:

1. Builds release binaries for 5 targets in parallel:
   - `x86_64-unknown-linux-gnu` (Linux x86_64)
   - `aarch64-unknown-linux-gnu` (Linux ARM64, via cross-compilation)
   - `x86_64-apple-darwin` (macOS Intel)
   - `aarch64-apple-darwin` (macOS Apple Silicon)
   - `x86_64-pc-windows-msvc` (Windows x86_64)
2. Packages each binary as `.tar.gz` (unix) or `.zip` (Windows) with SHA256 checksums
3. Extracts the release notes for that version from `CHANGELOG.md`
4. Creates a GitHub Release with the notes and all binaries attached
5. Marks it as a pre-release if the version is `0.x`

You don't need to manually create the release on GitHub — the workflow handles everything.

### Release artifacts

After a successful release, users will see download links like:

```
kite-v0.2.0-x86_64-unknown-linux-gnu.tar.gz
kite-v0.2.0-x86_64-unknown-linux-gnu.tar.gz.sha256
kite-v0.2.0-aarch64-apple-darwin.tar.gz
kite-v0.2.0-aarch64-apple-darwin.tar.gz.sha256
kite-v0.2.0-x86_64-pc-windows-msvc.zip
kite-v0.2.0-x86_64-pc-windows-msvc.zip.sha256
```

Users can verify downloads: `sha256sum -c kite-v0.2.0-x86_64-unknown-linux-gnu.tar.gz.sha256`

---

## Automated Release Workflow

The workflow at `.github/workflows/release.yml` triggers on any `v*` tag push.

**What it does:**
1. Checks out the code at the tagged commit
2. Uses [`ffurrer2/extract-release-notes`](https://github.com/ffurrer2/extract-release-notes) to parse `CHANGELOG.md` and extract the section matching the tag version
3. Creates a GitHub Release using `gh release create` with the extracted notes
4. Marks versions `0.x.x` as pre-release automatically

**How the extraction works:**
- The action looks for a `## [X.Y.Z]` header in `CHANGELOG.md` matching the tag
- It extracts everything between that header and the next `## [...]` header
- The extracted text becomes the release body on GitHub

This means the GitHub Release description always matches `CHANGELOG.md` — one source of truth, zero copy-paste.

---

## Versioning Policy

Kite follows [Semantic Versioning](https://semver.org/):

| Version bump | When |
|-------------|------|
| `0.x.y` → `0.x.(y+1)` | Bug fixes, small improvements |
| `0.x.y` → `0.(x+1).0` | New features, collectors, UI changes |
| `0.x.y` → `1.0.0` | First stable release (all Phase 1 complete, API considered stable) |

While in `0.x`, breaking changes can happen in minor versions. After `1.0.0`, breaking changes require a major version bump.

---

## Quick Reference

```bash
# Full release flow (after CHANGELOG.md and Cargo.toml are updated):
git add CHANGELOG.md Cargo.toml Cargo.lock
git commit -m "chore: release v0.2.0"
git tag -a v0.2.0 -m "v0.2.0"
git push origin main --follow-tags
# Done — workflow creates the GitHub Release automatically
```
