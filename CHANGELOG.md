# Changelog

<!-- markdownlint-disable MD024 -->

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

### Improved

### Fixed

## [0.2.1] - 2026-04-08

### Added

- **Configurable Health Check Prefixes** — Per-port and framework health endpoints can now opt into TCP-based checks with `grpc:`, `grpc://`, `ws:`, `ws://`, or `websocket:` prefixes.

### Improved

- **TUI Logs View** — The Logs tab now shows a live in-app activity stream for scans, tab changes, theme changes, and actions instead of a placeholder panel.
- **TUI Scrolling** — The main Ports table now renders a real viewport from the current scroll offset so large result sets track selection and mouse clicks correctly.

### Fixed

- **Port Conflicts** — Conflict detection now evaluates raw listeners before scan deduplication, restoring correct detection for multiple processes on the same protocol/port.
- **Health Checks** — gRPC and WebSocket health paths are now selected by the scan pipeline instead of being dead-end helper functions.
- **Tunnel Detection** — Added Tailscale Funnel detection to match the documented Phase 2 support matrix.
- **Resource History** — Enforced the intended 1-second sample interval so sparkline history reflects a stable time window.
- **Mouse Toggle** — Toggling mouse mode now updates terminal mouse capture state at runtime.

## [0.2.0] - 2026-04-07

### Added

- **Free Port Finder** — New `portforge free <start>` command to find available ports. Supports `--count` for finding multiple free ports.
- **Port Conflict Detection** — New `portforge conflicts` command to detect multiple processes on the same port with resolution suggestions.
- **gRPC Health Check** — TCP connection-based health check for gRPC services.
- **WebSocket Health Check** — TCP connection-based health check for WebSocket services.
- **Process Resource History** — Tracks CPU and memory usage over time per process. Sparkline graphs shown in port detail view and Processes tab.
- **Tab-Based Views** — New tab bar with Ports, Processes, Docker, and Logs views. Navigate with `Tab`/`Shift+Tab`.
- **Custom Color Themes** — 5 built-in themes: dark (default), light, solarized, nord, dracula. Cycle with `T` key.
- **TUI Mouse Support** — Click to select rows, scroll wheel to navigate. Toggle with `m` key.

### Improved

- **TUI Architecture** — Refactored Theme from static constants to instance-based design for dynamic theme switching.
- **TUI Layout** — Added tab bar between header and content area.
- **Detail View** — Now shows sparkline graphs for CPU and memory history with avg/peak stats.
- **Processes Tab** — New dedicated view sorted by CPU usage with sparkline indicators.
- **Docker Tab** — New dedicated view showing only Docker containers.
- **Help Overlay** — Updated with new keybindings for tabs, themes, and mouse toggle.
- **Status Bar** — Updated to show Tab, T (theme), and m (mouse) key hints.

## [0.1.2] - 2026-04-07

### Added

- Marketing website

### Improved

- TUI and web dashboard to match marketing website

### Fixed

## [0.1.1] - 2026-04-06

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
