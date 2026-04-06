use clap::Parser;
use portforge::cli::{Cli, Commands, ExportFormat};
use portforge::config::PortForgeConfig;
use portforge::error::Result;
use portforge::export;
use portforge::process;
use portforge::scanner;
use portforge::tui::app::App;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize logging
    let filter = if cli.verbose {
        EnvFilter::new("debug")
    } else {
        EnvFilter::from_default_env()
    };
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .init();

    // Disable colors if requested (NO_COLOR convention)
    if cli.no_color {
        // TODO: Audit that the environment access only happens in single-threaded code.
        unsafe { std::env::set_var("NO_COLOR", "1") };
    }

    // Load config
    let config = PortForgeConfig::load().unwrap_or_default();

    let show_all = cli.all || config.general.show_all;

    match cli.command {
        // No subcommand → launch TUI or table output
        None => {
            if cli.json {
                let entries = scanner::scan_ports(&config, show_all).await?;
                println!("{}", export::to_json(&entries, true)?);
            } else if cli.csv {
                let entries = scanner::scan_ports(&config, show_all).await?;
                print!("{}", export::to_csv(&entries));
            } else if atty::is(atty::Stream::Stdout) {
                // Interactive terminal → launch TUI
                let mut app = App::new(config, show_all);
                app.run().await?;
            } else {
                // Piped output → plain table
                let entries = scanner::scan_ports(&config, show_all).await?;
                println!("{}", export::to_table(&entries));
            }
        }

        Some(Commands::Inspect { port }) => {
            let entries = scanner::scan_ports(&config, true).await?;
            if let Some(entry) = entries.iter().find(|e| e.port == port) {
                if cli.json {
                    println!("{}", serde_json::to_string_pretty(entry)?);
                } else {
                    export::print_inspection(entry);
                }
            } else {
                eprintln!("⚠ No process found listening on port {}", port);
                std::process::exit(1);
            }
        }

        Some(Commands::Kill { port, force }) => {
            let entries = scanner::scan_ports(&config, true).await?;
            if let Some(entry) = entries.iter().find(|e| e.port == port) {
                process::kill_process(entry, force)?;
                println!(
                    "✓ {} PID {} on port {}",
                    if force { "Force killed" } else { "Stopped" },
                    entry.pid,
                    port
                );
            } else {
                eprintln!("⚠ No process found listening on port {}", port);
                std::process::exit(1);
            }
        }

        Some(Commands::Clean { dry_run }) => {
            let entries = scanner::scan_ports(&config, true).await?;
            let results = process::clean_orphans(&entries, dry_run)?;

            if results.is_empty() {
                println!("✓ No orphaned or zombie processes found.");
            } else {
                for result in &results {
                    let icon = if result.success { "✓" } else { "✗" };
                    println!(
                        "{} {} {} ({}) on port {}",
                        icon, result.action, result.process_name, result.pid, result.port
                    );
                }
                println!(
                    "\n{} process(es) {}.",
                    results.len(),
                    if dry_run {
                        "would be cleaned"
                    } else {
                        "cleaned"
                    }
                );
            }
        }

        Some(Commands::Watch { interval }) => {
            let mut app = App::new(config, show_all);
            app.set_refresh_interval(interval);
            app.run().await?;
        }

        Some(Commands::Ps) => {
            let entries = scanner::scan_ports(&config, show_all).await?;
            if cli.json {
                println!("{}", export::to_json(&entries, true)?);
            } else {
                println!("{}", export::to_table(&entries));
            }
        }

        Some(Commands::Export { format, output }) => {
            let entries = scanner::scan_ports(&config, show_all).await?;
            let content = match format {
                ExportFormat::Json => export::to_json(&entries, true)?,
                ExportFormat::Csv => export::to_csv(&entries),
            };

            match output {
                Some(path) => {
                    std::fs::write(&path, &content)?;
                    println!("✓ Exported {} entries to {}", entries.len(), path);
                }
                None => print!("{}", content),
            }
        }

        #[cfg(feature = "web")]
        Some(Commands::Serve { port, bind }) => {
            println!(
                "⚡ Starting PortForge web dashboard on http://{}:{}",
                bind, port
            );
            portforge::web::server::start_server(&bind, port, config).await?;
        }

        Some(Commands::InitConfig) => {
            let path = PortForgeConfig::write_default()?;
            println!("✓ Default configuration written to {}", path.display());
        }
    }

    Ok(())
}
