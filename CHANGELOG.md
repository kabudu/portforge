# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- **Rust 2024 Edition** — Migrated to the latest Rust edition and bumped the minimum supported `rust-version` to 1.85.
- **Tunnel Detection** — New `tunnel.rs` module for detecting ngrok, cloudflared, localtunnel, and SSH reverse tunnels with public URL extraction.
- **TUI Scrolling** — Implemented table auto-scroll and manual navigation (Home/End/g/G) using `table_scroll_offset`.
- **Tunnel Column** — Added dedicated "Tunnel" column to both the TUI dashboard and tabular exports (CSV/Table).

### Improved

- **TUI Responsiveness** — Refactored event loop to use `async` key handling, enabling immediate UI updates on toggle (e.g., 'a' for all ports), refresh ('r'), and process kill.
- **Health Checks** — Added concurrency limiting via `tokio::sync::Semaphore` to prevent resource exhaustion during scans.
- **Docker Integration** — Replaced silent fallback/panics with proper warning logs when the Docker daemon is unreachable.
- **Kill Process** — Re-architected `kill_process` with exponential backoff retries and post-signal existence verification.
- **Tests** — Expanded test coverage by 22 cases across scanner, models, process, and tunnel logic (51 total tests).
- **Documentation** — Added comprehensive doc-comments for core scanning and status determination functions.

### Fixed

- **Dev Port Detection** — Resolved issue where certain projects (like "deliberium") were hidden by enabling CWD (Current Working Directory) retrieval.
- **TUI Navigation** — Fixed incorrect 'a' key mapping (previously bound to 'T') for toggling the dev/all port filter.
- **CI Workflow** — Fixed project formatting (fmt) and addressed multiple clippy lints to unblock automated checks.
- **Metrics** — Resolved CPU percentage reporting 0.0% by enforcing a global thread-safe state cache spanning refresh cycles.
- **Metrics** — Prevented memory sizes formatting as negative zeros (-0.0) in aggregate floats.
- **Docker** — Identified Docker containers correctly map natively to "Healthy" overriding default fallback.
- **Thread Safety** — Enforced proper `Send` bound drops across Web API health-probe polling asynchronous logic.

## [0.1.0] - 2026-04-04

### Added

- **Core Scanner** — Cross-platform port scanning via `listeners` crate with PID enrichment from `sysinfo`
- **Project Detection** — Auto-detects 20+ languages and 40+ frameworks (Rust, Node.js, Python, Go, Ruby, Java, PHP, Elixir, Swift, Dart, Zig, and more)
- **Git Integration** — Branch name and dirty status via `git2`
- **Docker Integration** — Container name, image, and compose project via `bollard` (feature-gated)
- **Health Checks** — HTTP probes with framework-aware default endpoints and configurable timeout
- **Interactive TUI** — Full ratatui-based terminal UI with:
  - Vim-style navigation (j/k/gg/G)
  - Column sorting (1-8 keys)
  - Search and filtering (/)
  - Port detail view (Enter/d)
  - Process tree view (t)
  - Kill confirmation dialog (K)
  - Help overlay (?)
  - Auto-refresh in watch mode
- **Web Dashboard** — Axum + HTMX web interface (behind `--features web`) with:
  - Real-time stats cards
  - Auto-refreshing port table (3s interval)
  - Port detail modal
  - One-click kill with confirmation
  - Client-side search
  - Dark glassmorphism theme
- **CLI Commands** — `inspect`, `kill`, `clean`, `watch`, `ps`, `export`, `serve`, `init-config`
- **Export** — JSON, CSV, and pretty table output formats
- **Configuration** — TOML config file at `~/.config/portforge.toml` with framework endpoints, custom detectors, and per-port overrides
- **Safe Cleanup** — Orphan/zombie detection with `--dry-run` preview
- **CI/CD** — GitHub Actions for lint, test (cross-platform), and release (cross-compiled binaries)
- **Documentation** — README, CONTRIBUTING, ROADMAP, SECURITY, CODE_OF_CONDUCT, CHANGELOG
