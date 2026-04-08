<!-- markdownlint-disable MD033 MD036 MD041 -->

<div align="center">

<img src="docs/assets/logo.png" alt="PortForge logo" width="120" />

# PortForge

**Modern cross-platform port inspector & manager for developers**

[![CI](https://github.com/kabudu/portforge/actions/workflows/ci.yml/badge.svg)](https://github.com/kabudu/portforge/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.85%2B-orange.svg)](https://www.rust-lang.org)

_Know what's running on your ports — instantly, with rich developer context._

</div>

---

## ✨ Features

- 🚀 **Blazing fast** — <50ms cold start, single static binary (~5-8 MB)
- 🖥️ **Beautiful TUI** — Full interactive terminal UI with vim keybindings
- 🌐 **Web Dashboard** — Optional HTMX-powered web interface (feature flag)
- 🔍 **Project Detection** — Auto-detects 20+ languages & 40+ frameworks
- 🔀 **Git Integration** — Shows branch name and dirty status
- 🐳 **Docker/Podman** — Native container port mapping via Bollard API
- 🏥 **Health Checks** — HTTP probes with framework-aware endpoints
- 🌲 **Process Trees** — Drill down into parent/child process hierarchies
- 📊 **Resource Monitoring** — CPU%, memory, uptime per process
- 🧹 **Safe Cleanup** — Orphan/zombie detection with dry-run preview
- 📤 **Export** — JSON, CSV, and pretty table output
- ⚙️ **Configurable** — TOML config file for custom behavior
- 💻 **Cross-platform** — Linux, macOS, and Windows

## 📦 Installation

### From Source (Cargo)

```bash
cargo install portforge
```

### From GitHub Releases

Download the latest binary for your platform from [Releases](https://github.com/kabudu/portforge/releases).

```bash
# macOS / Linux
OS=$(uname -s | tr '[:upper:]' '[:lower:]')
[ "$OS" = "darwin" ] && OS="macos"
ARCH=$(uname -m)
[ "$ARCH" = "arm64" ] && ARCH="aarch64"
curl -L "https://github.com/kabudu/portforge/releases/latest/download/portforge-${OS}-${ARCH}.tar.gz" | tar xz
sudo mv portforge /usr/local/bin/
```

### Build from Source

```bash
git clone https://github.com/kabudu/portforge.git
cd portforge
cargo build --release

# With web dashboard
cargo build --release --features web
```

## 🚀 Quick Start

```bash
# Launch interactive TUI (default)
portforge

# Show all ports (including non-dev)
portforge --all

# Inspect a specific port
portforge inspect 3000

# Kill a process on a port
portforge kill 3000
portforge kill 3000 --force

# Clean orphaned processes
portforge clean --dry-run
portforge clean

# Live watch mode
portforge watch

# Export as JSON
portforge ps --json

# Launch web dashboard (requires --features web)
portforge serve --port 9090
```

## 🖥️ TUI Keybindings

| Key           | Action                           |
| ------------- | -------------------------------- |
| `j` / `↓`     | Move down                        |
| `k` / `↑`     | Move up                          |
| `g` / `G`     | Go to top / bottom               |
| `Enter` / `d` | View port details                |
| `K`           | Kill process (with confirmation) |
| `t`           | Process tree view                |
| `/`           | Search / filter                  |
| `a`           | Toggle all / dev ports           |
| `1`-`8`       | Sort by column                   |
| `Tab`         | Next tab                         |
| `Shift+Tab`   | Previous tab                     |
| `T`           | Cycle theme                      |
| `m`           | Toggle mouse support             |
| `?`           | Help overlay                     |
| `q` / `Esc`   | Quit / go back                   |

While the search bar is open, `j` and `k` continue moving through the filtered result set.

## 🌐 Web Dashboard

Enable the web dashboard with the `web` feature flag:

```bash
cargo build --release --features web
portforge serve --port 9090
```

The dashboard provides:

- 📊 Real-time stats cards (ports, healthy, docker, memory)
- 📋 Auto-refreshing port table (HTMX, every 3s)
- 🔍 Click-to-inspect port details
- 🔴 One-click kill with confirmation
- 🔎 Client-side search filtering
- 🌑 Beautiful dark glassmorphism theme

## ⚙️ Configuration

Generate a default config file:

```bash
portforge init-config
```

Configuration is stored at `~/.config/portforge.toml`:

```toml
[general]
refresh_interval = 2          # Watch mode refresh (seconds)
show_all = false              # Show non-dev ports by default
docker_enabled = true         # Enable Docker integration
health_checks_enabled = true  # Enable HTTP health probes
theme = "dark"                # Color theme

[health]
timeout_ms = 2000             # Health check timeout

[health.framework_endpoints]
"next.js" = "/api/health"
"express" = "/health"
"rails" = "/up"
"spring" = "/actuator/health"
"django" = "/health/"
"my-grpc-service" = "grpc:"          # TCP connect check
"my-websocket-app" = "ws:"           # TCP connect check

# Per-port overrides
# [ports.3000]
# label = "My Frontend"
# health_endpoint = "/api/status"
# hidden = false

# [ports.50051]
# label = "Local gRPC API"
# health_endpoint = "grpc:"
# hidden = false

# [ports.3001]
# label = "Socket Server"
# health_endpoint = "ws:"
# hidden = false
```

Health endpoint prefixes:

- `grpc:` or `grpc://` uses a TCP connection check instead of HTTP for gRPC-style services.
- `ws:`, `ws://`, or `websocket:` uses a TCP connection check instead of HTTP for WebSocket-style services.
- Plain values like `/health` continue to use normal HTTP probing.

## 🏗️ Architecture

```text
portforge/
├── src/
│   ├── main.rs          # CLI entry point
│   ├── lib.rs           # Library root
│   ├── cli.rs           # Clap CLI definitions
│   ├── scanner.rs       # Core port scanning & enrichment
│   ├── models.rs        # Data structures
│   ├── process.rs       # Kill, clean, process tree
│   ├── project.rs       # Framework detection (20+ languages)
│   ├── docker.rs        # Bollard Docker/Podman integration
│   ├── git.rs           # git2 branch/dirty detection
│   ├── health.rs        # HTTP health probes
│   ├── config.rs        # TOML configuration
│   ├── export.rs        # JSON/CSV/table output
│   ├── error.rs         # Error types (thiserror)
│   ├── tui/             # Ratatui interactive TUI
│   └── web/             # Axum + HTMX web dashboard
└── tests/               # Integration tests
```

**Key dependencies:**

- [`ratatui`](https://ratatui.rs) — Terminal UI framework
- [`crossterm`](https://github.com/crossterm-rs/crossterm) — Terminal backend
- [`sysinfo`](https://github.com/GuillaumeGomez/sysinfo) — System/process info
- [`listeners`](https://crates.io/crates/listeners) — Port→PID mapping
- [`bollard`](https://github.com/fussybeaver/bollard) — Docker API client
- [`git2`](https://github.com/rust-lang/git2-rs) — Git integration
- [`axum`](https://github.com/tokio-rs/axum) — Web framework (optional)

## 🤝 Contributing

Contributions are welcome! Please read [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## 🗺️ Roadmap

See [ROADMAP.md](ROADMAP.md) for the project roadmap.

## 📜 License

This project is licensed under the MIT License — see [LICENSE](LICENSE) for details.

## 🔒 Security

Please see [SECURITY.md](SECURITY.md) for our security policy.
