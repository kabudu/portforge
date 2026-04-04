# Contributing to PortForge

Thank you for your interest in contributing to PortForge! This document provides guidelines and information for contributors.

## 🚀 Getting Started

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) 1.75 or later
- Git
- Docker (optional, for Docker integration testing)

### Development Setup

```bash
# Clone the repository
git clone https://github.com/kabudu/portforge.git
cd portforge

# Build the project
cargo build

# Run tests
cargo test

# Run with verbose logging
RUST_LOG=debug cargo run

# Build with web dashboard
cargo build --features web
```

### Running Locally

```bash
# Launch the TUI
cargo run

# Launch with all ports visible
cargo run -- --all

# Launch the web dashboard
cargo run --features web -- serve --port 9090
```

## 📋 How to Contribute

### Reporting Bugs

1. **Search existing issues** to avoid duplicates.
2. Use the [Bug Report](https://github.com/kabudu/portforge/issues/new?template=bug_report.md) template.
3. Include:
   - OS and version
   - Rust version (`rustc --version`)
   - Steps to reproduce
   - Expected vs actual behavior
   - Relevant logs (`RUST_LOG=debug portforge 2>&1`)

### Suggesting Features

1. Use the [Feature Request](https://github.com/kabudu/portforge/issues/new?template=feature_request.md) template.
2. Describe the use case and expected behavior.
3. Explain why this would benefit other users.

### Submitting Pull Requests

1. **Fork** the repository and create a feature branch:
   ```bash
   git checkout -b feature/my-awesome-feature
   ```

2. **Make your changes** following the code style guidelines below.

3. **Add tests** for new functionality.

4. **Run the full test suite**:
   ```bash
   cargo test
   cargo test --features web
   cargo clippy -- -D warnings
   cargo fmt --check
   ```

5. **Commit** with a clear message following [Conventional Commits](https://www.conventionalcommits.org/):
   ```
   feat: add tunnel detection for ngrok
   fix: handle Docker socket permission errors gracefully
   docs: update configuration examples
   test: add health check timeout tests
   ```

6. **Push** and open a Pull Request against `main`.

## 🎨 Code Style Guidelines

### Rust

- Follow standard Rust conventions and idioms.
- Run `cargo fmt` before committing.
- Address all `cargo clippy` warnings.
- Use `thiserror` for error types, `anyhow` for ad-hoc errors.
- Prefer `tracing` over `println!` for debug output.
- Document public APIs with `///` doc comments.
- Use descriptive variable names — avoid single letters except in iterators.

### Git

- Keep commits atomic and focused.
- Write clear commit messages.
- Rebase (don't merge) to keep history clean.

### Testing

- Write unit tests for new functions.
- Write integration tests for new CLI commands.
- Aim for tests that don't require specific system state (mock where possible).
- Docker-dependent tests should gracefully skip when Docker is unavailable.

## 🏗️ Project Structure

```
src/
├── main.rs          # Entry point — keep thin
├── lib.rs           # Module declarations
├── cli.rs           # CLI argument definitions
├── scanner.rs       # Core scanning logic
├── models.rs        # Data structures
├── process.rs       # Process management
├── project.rs       # Framework detection
├── docker.rs        # Docker integration
├── git.rs           # Git integration
├── health.rs        # Health checks
├── config.rs        # Configuration
├── export.rs        # Output formatting
├── error.rs         # Error types
├── tui/             # Terminal UI
│   ├── app.rs       # State & event loop
│   ├── ui.rs        # Rendering
│   ├── widgets.rs   # Custom widgets
│   └── theme.rs     # Colors & styles
└── web/             # Web dashboard (feature-gated)
    ├── server.rs    # Server setup
    ├── handlers.rs  # Route handlers
    └── static/      # CSS/JS assets
```

## 📝 Adding a New Framework Detector

1. Open `src/project.rs`
2. Add your framework to the `detectors` list in `detect_in_dir()`
3. Add framework name mapping in `capitalize_framework()`
4. Add a health endpoint in `src/config.rs` → `HealthConfig::default()`
5. Add a test in `src/project.rs` → `mod tests`

## 🔄 Release Process

1. Update version in `Cargo.toml`
2. Update `CHANGELOG.md`
3. Create a PR titled `release: v0.X.Y`
4. After merge, tag: `git tag v0.X.Y && git push --tags`
5. GitHub Actions will build and publish the release

## 📜 License

By contributing, you agree that your contributions will be licensed under the MIT License.

## 💬 Questions?

Feel free to open a [Discussion](https://github.com/kabudu/portforge/discussions) or reach out via Issues.

Thank you for helping make PortForge better! ⚡
