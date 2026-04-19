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

### Using the release script (recommended)

The `scripts/release.ps1` script automates the entire release process:

```powershell
# Preview what would happen (no changes made)
./scripts/release.ps1 patch -DryRun

# Cut a patch release (local only — creates commit + tag)
./scripts/release.ps1 patch

# Cut a minor release and push immediately
./scripts/release.ps1 minor -Push

# Cut a major release
./scripts/release.ps1 major -Push
```

The script performs these steps automatically:
1. Reads current version from `Cargo.toml`
2. Bumps the specified semver component (major, minor, or patch)
3. **Preflight checks**: clean working tree, on `main` branch, tag available, changelog has entries, tests pass, clippy clean
4. Updates `Cargo.toml` version
5. Runs `cargo check` to update `Cargo.lock`
6. Stamps the `[Unreleased]` section in `CHANGELOG.md` with version and date
7. Updates comparison links at the bottom of `CHANGELOG.md`
8. Commits with message `chore: release vX.Y.Z`
9. Creates annotated tag `vX.Y.Z`
10. Optionally pushes to origin (with `-Push` flag)

### Manual release process

If you prefer to release manually:

#### Step 1 — Finalize the changelog

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

#### Step 2 — Bump the version in Cargo.toml

```toml
[package]
version = "0.2.0"
```

#### Step 3 — Commit, tag, and push

```bash
git add CHANGELOG.md Cargo.toml Cargo.lock
git commit -m "chore: release v0.2.0"
git tag -a v0.2.0 -m "v0.2.0"
git push origin main --follow-tags
```

### Step 4 — The release workflow takes over

Once you push a `v*` tag, the GitHub Actions release workflow (`.github/workflows/release.yml`) automatically:

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
6. **Publishes to crates.io** via `cargo publish`

You don't need to manually create the release on GitHub or publish to crates.io — the workflow handles everything.

---

## Publishing to crates.io

The release workflow automatically publishes to [crates.io](https://crates.io/crates/kite) after creating the GitHub Release.

### Setup (one-time)

1. Create an API token at [crates.io/settings/tokens](https://crates.io/settings/tokens) with `publish-update` scope
2. Add it as a repository secret named `CARGO_REGISTRY_TOKEN` in GitHub Settings → Secrets → Actions

### What gets published

- The `kite` crate with all metadata from `Cargo.toml` (description, license, repository, keywords, categories)
- Only the source code — binary artifacts are separate (GitHub Releases)
- Users can install with: `cargo install kite`

### Manual publish (if needed)

```bash
# Dry run to verify
cargo publish --dry-run

# Publish
cargo publish
```

---

## Release Artifacts

After a successful release, users get:

**GitHub Release** (binary downloads):
```
kite-v0.2.0-x86_64-unknown-linux-gnu.tar.gz
kite-v0.2.0-x86_64-unknown-linux-gnu.tar.gz.sha256
kite-v0.2.0-aarch64-apple-darwin.tar.gz
kite-v0.2.0-aarch64-apple-darwin.tar.gz.sha256
kite-v0.2.0-x86_64-pc-windows-msvc.zip
kite-v0.2.0-x86_64-pc-windows-msvc.zip.sha256
```

Users can verify downloads: `sha256sum -c kite-v0.2.0-x86_64-unknown-linux-gnu.tar.gz.sha256`

**crates.io** (source install):
```bash
cargo install kite
```

---

## Automated Release Workflow

The workflow at `.github/workflows/release.yml` triggers on any `v*` tag push.

**What it does:**
1. Checks out the code at the tagged commit
2. Builds cross-platform release binaries (5 targets)
3. Packages with SHA256 checksums
4. Uses [`ffurrer2/extract-release-notes`](https://github.com/ffurrer2/extract-release-notes) to parse `CHANGELOG.md` and extract the section matching the tag version
5. Creates a GitHub Release using `gh release create` with the extracted notes
6. Marks versions `0.x.x` as pre-release automatically
7. Publishes to crates.io via `cargo publish --no-verify`

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

```powershell
# Recommended: use the release script
./scripts/release.ps1 patch -Push        # bug fix
./scripts/release.ps1 minor -Push        # new features
./scripts/release.ps1 major -Push        # breaking changes

# Or manually:
git add CHANGELOG.md Cargo.toml Cargo.lock
git commit -m "chore: release v0.2.0"
git tag -a v0.2.0 -m "v0.2.0"
git push origin main --follow-tags
# Done — workflow builds binaries, creates GitHub Release, publishes to crates.io
```
