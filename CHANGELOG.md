# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
