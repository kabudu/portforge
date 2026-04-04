use clap::{Parser, Subcommand, ValueEnum};

/// PortForge — Modern cross-platform port inspector & manager for developers
///
/// A developer-first tool that shows what's running on your ports
/// with project detection, Docker integration, health checks, and more.
#[derive(Parser, Debug)]
#[command(
    name = "portforge",
    author = "Kamba",
    version,
    about = "⚡ Modern port inspector & manager for developers",
    long_about = "PortForge is a fast, cross-platform port inspector and manager.\n\n\
                  It enriches port listings with project detection, framework info,\n\
                  git branch status, Docker container mapping, and health checks.\n\n\
                  Run without arguments to launch the interactive TUI."
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Show all ports (not just dev projects)
    #[arg(short, long, global = true)]
    pub all: bool,

    /// Output as JSON
    #[arg(long, global = true)]
    pub json: bool,

    /// Output as CSV
    #[arg(long, global = true)]
    pub csv: bool,

    /// Disable colored output
    #[arg(long, global = true)]
    pub no_color: bool,

    /// Enable verbose logging (RUST_LOG=debug)
    #[arg(short, long, global = true)]
    pub verbose: bool,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Inspect a specific port in detail
    Inspect {
        /// Port number to inspect
        port: u16,
    },

    /// Kill the process on a given port
    Kill {
        /// Port number to kill
        port: u16,

        /// Force kill (SIGKILL instead of SIGTERM)
        #[arg(short, long)]
        force: bool,
    },

    /// Clean up orphaned and zombie port processes
    Clean {
        /// Preview what would be cleaned without actually killing
        #[arg(short, long)]
        dry_run: bool,
    },

    /// Launch live-updating watch mode
    Watch {
        /// Refresh interval in seconds
        #[arg(short, long, default_value = "2")]
        interval: u64,
    },

    /// Show htop-style process list filtered to listening ports
    Ps,

    /// Export port data
    Export {
        /// Export format
        #[arg(short, long, default_value = "json")]
        format: ExportFormat,

        /// Output file (stdout if not specified)
        #[arg(short, long)]
        output: Option<String>,
    },

    /// Launch the web dashboard
    #[cfg(feature = "web")]
    Serve {
        /// Port for the web dashboard
        #[arg(short, long, default_value = "9090")]
        port: u16,

        /// Bind address
        #[arg(short, long, default_value = "127.0.0.1")]
        bind: String,
    },

    /// Generate default configuration file
    InitConfig,
}

#[derive(Debug, Clone, ValueEnum)]
pub enum ExportFormat {
    Json,
    Csv,
}
