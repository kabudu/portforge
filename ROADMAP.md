# PortForge Roadmap

This document outlines the planned development phases for PortForge.

## ✅ Phase 1: MVP (v0.1.0) — Current

> Core functionality that makes PortForge immediately useful.

- [x] Cross-platform port scanning (Linux, macOS, Windows)
- [x] Process enrichment (PID, name, command, CPU, memory, uptime)
- [x] Project/framework detection (20+ languages, 40+ frameworks)
- [x] Git integration (branch, dirty status)
- [x] Docker/Podman integration (container name, image, compose project)
- [x] HTTP health probes (framework-aware endpoints)
- [x] Beautiful interactive TUI (ratatui)
  - [x] Vim-style navigation (j/k/gg/G)
  - [x] Search and filtering
  - [x] Sort by any column
  - [x] Port detail view
  - [x] Process tree view
  - [x] Kill confirmation dialog
  - [x] Help overlay
- [x] Web dashboard (Axum + HTMX, feature-gated)
  - [x] Real-time stats cards
  - [x] Auto-refreshing port table
  - [x] Port detail modal
  - [x] One-click kill
  - [x] Search filtering
  - [x] Dark glassmorphism theme
- [x] CLI commands (inspect, kill, clean, watch, ps, export)
- [x] JSON/CSV export
- [x] TOML configuration file
- [x] Safe cleanup with dry-run preview
- [x] CI/CD pipeline (GitHub Actions)
- [x] Cross-platform release binaries

---

## 🔜 Phase 2: Intelligence (v0.2.0)

> Smarter detection, better UX, and developer workflow integration.

- [ ] Tunnel detection (ngrok, cloudflared, localtunnel, Tailscale Funnel)
- [ ] Auto-suggest free ports (`portforge free 3000`)
- [ ] Port conflict detection and resolution suggestions  
- [ ] gRPC health check support
- [ ] WebSocket health check support
- [ ] Process resource history (sparkline graphs in TUI)
- [ ] TUI mouse support (click to select, scroll)
- [ ] Custom color themes (light mode, solarized, etc.)
- [ ] Tab-based TUI views (ports, processes, docker, logs)

---

## 🔮 Phase 3: Ecosystem (v0.3.0)

> Plugin system and integrations with the broader developer ecosystem.

- [ ] Plugin system for custom detectors (dynamic `.so`/`.dll` loading)
- [ ] VS Code extension (calls portforge binary for port info)
- [ ] JetBrains plugin
- [ ] `just` / `cargo-make` / `Makefile` integration
- [ ] Notification system (port started, port died, health degraded)
  - [ ] macOS native notifications
  - [ ] Linux D-Bus notifications
  - [ ] Windows toast notifications
- [ ] Kubernetes pod port-forwarding awareness
- [ ] SSH tunnel detection and management

---

## 🌟 Phase 4: Distribution (v1.0.0)

> Stable release with wide distribution and polish.

- [ ] Publish to crates.io
- [ ] Homebrew formula (`brew install portforge`)
- [ ] Scoop manifest (Windows)
- [ ] APT/RPM packages
- [ ] Nix package
- [ ] AUR package (Arch Linux)
- [ ] Shell completions (bash, zsh, fish, PowerShell)
- [ ] Man page generation
- [ ] Comprehensive documentation site
- [ ] Performance benchmarks and optimization
- [ ] Accessibility improvements (screen reader support)

---

## 💡 Ideas & Wishlist

These are ideas that may be implemented if there's community interest:

- **Port groups** — Group related ports (e.g., frontend + backend + database)
- **Port bookmarks** — Save frequently-used port configurations
- **History** — Track port usage over time
- **Alerts** — Configurable alerts for port events (Slack, Discord, email)
- **Remote monitoring** — Monitor ports on remote machines via SSH
- **API mode** — Run as a daemon with a REST API for other tools
- **Dashboard sharing** — Generate shareable snapshots of port state

---

## 🗳️ Have a Suggestion?

We'd love to hear from you! Open a [Feature Request](https://github.com/kabudu/portforge/issues/new) or start a [Discussion](https://github.com/kabudu/portforge/discussions).
