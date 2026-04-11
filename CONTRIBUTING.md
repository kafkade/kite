# Contributing to Kite

Welcome! Kite is a Rust TUI system resource monitor built for performance and usability. If you're interested in contributing, you're in the right place. For an overview of the project, see the [README](README.md).

## Finding Something to Work On

Looking for a place to start? Here are some great ways to find work:

- Look for issues labeled **"good first issue"** or **"help wanted"** - these are ideal for newcomers
- Check [PLAN.md](PLAN.md) for the implementation roadmap and upcoming features
- Read [docs/SPEC.md](docs/SPEC.md) for the full specification and design decisions

## Before You Start

Please follow these guidelines depending on the type of contribution:

- **Bug fixes**: Open an issue first if one doesn't already exist. This helps us track the problem and ensures you're not duplicating work.
- **New features**: Open an issue to discuss the approach before writing code. This prevents wasted effort and ensures the feature aligns with the project vision.
- **Documentation**: Pull requests welcome without prior discussion. Fix typos, clarify instructions, or add examples directly.

## Development Setup

### Prerequisites

- **Rust 1.85+** (install via [https://rustup.rs/](https://rustup.rs/))
- **Git**

### Getting the Code

```bash
git clone https://github.com/kafkade/kite.git
cd kite
cargo build
cargo test
```

## Development Workflow

Use these commands for development:

```bash
cargo build              # Build the project
cargo test               # Run all tests
cargo clippy -- -D warnings  # Lint (must pass)
cargo fmt                # Format code
cargo run                # Launch the TUI
cargo run -- --interval 500   # Launch with custom tick rate
```

## Code Standards

When contributing code, please follow these standards:

- **Rust Edition 2024** with **clippy::pedantic** lint level
- Use **thiserror** for library/domain errors, **anyhow** at the binary boundary
- Prefer **tokio::sync::watch** channels for collector-to-UI data flow
- Each collector should be independently testable with mock data
- Document all public APIs with `///` doc comments
- Keep functions small and modules focused on a single responsibility
- **Never use `unwrap()`** in library code - use proper error handling

## Testing

Before submitting a pull request, ensure:

- ✅ All new functionality has tests
- ✅ All existing tests pass: `cargo test`
- ✅ Clippy passes with no warnings: `cargo clippy -- -D warnings`
- ✅ Code is formatted: `cargo fmt -- --check`

## Submitting a Pull Request

### Fork and Branch

1. Fork the repository on GitHub
2. Clone your fork locally: `git clone https://github.com/YOUR-USERNAME/kite.git`
3. Create a feature branch from main: `git checkout -b feat/my-feature`
4. Use descriptive branch names:
   - `fix/cpu-overflow`
   - `feat/gpu-collector`
   - `docs/install-guide`
   - `refactor/ring-buffer`

### Making Changes

1. Make your changes in focused, logical commits
2. Follow the code standards above
3. Add or update tests as needed
4. Run the full check suite before pushing:
   ```bash
   cargo test && cargo clippy -- -D warnings && cargo fmt -- --check
   ```

### Opening the PR

1. Push your branch to your fork
2. Open a pull request against `kafkade/kite` main branch
3. Fill out the PR template completely
4. Reference any related issues (e.g., "Closes #42")
5. Keep PRs focused - one feature or fix per PR

### Commit Messages

We follow **Conventional Commits** format for PR titles:

- `feat: add GPU temperature collector`
- `fix: correct CPU percentage calculation overflow`
- `docs: update installation instructions`
- `ci: add Windows to test matrix`
- `refactor: extract ring buffer into standalone module`
- `test: add integration tests for network collector`

### What Happens Next

1. **CI will run automatically** to check tests, clippy, and formatting
2. **Maintainer review** - we'll review your code and may ask for changes
3. **Make adjustments** - push additional commits to address feedback
4. **Approval & merge** - once approved, your PR will be squash-merged into main
5. **Recognition** - your contribution will be noted in the changelog

## Code of Conduct

This project follows the [Contributor Covenant Code of Conduct](CODE_OF_CONDUCT.md). By participating, you are expected to uphold this code. Please report unacceptable behavior to the project maintainers.

## Security

If you discover a security vulnerability, please do **NOT** open a public issue. Instead, see [SECURITY.md](SECURITY.md) for responsible disclosure instructions.

## Questions?

- **Open a GitHub issue** for bug reports or feature discussions
- **Check existing issues** before creating new ones to avoid duplicates
- **Be respectful and constructive** in all communications

---

Thank you for contributing to Kite! Your efforts help make this project better for everyone. 🚀
