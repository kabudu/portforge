use clap::Parser;
use portforge::cli::{Cli, Commands, ExportFormat};
use portforge::config::PortForgeConfig;
use portforge::error::{PortForgeError, Result};
use portforge::export;
use portforge::models::{PortEntry, Protocol};
use portforge::port_utils;
use portforge::process;
use portforge::scanner;
use portforge::tui::app::App;
use std::io::IsTerminal;
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
        .with_ansi(!cli.no_color)
        .init();

    // Load config
    let config = PortForgeConfig::load()?;

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
            } else if std::io::stdout().is_terminal() {
                // Interactive terminal → launch TUI
                let mut app = App::new(config, show_all);
                app.run().await?;
            } else {
                // Piped output → plain table
                let entries = scanner::scan_ports(&config, show_all).await?;
                println!("{}", export::to_table(&entries));
            }
        }

        Some(Commands::Inspect {
            port,
            pid,
            protocol,
        }) => {
            let entries = scanner::scan_ports(&config, true).await?;
            if let Some(entry) = select_entry(&entries, port, protocol, pid)? {
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

        Some(Commands::Kill {
            port,
            pid,
            protocol,
            force,
        }) => {
            let entries = scanner::scan_ports(&config, true).await?;
            if let Some(entry) = select_entry(&entries, port, protocol, pid)? {
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

        Some(Commands::Free { start, count }) => {
            let free_ports = port_utils::find_free_ports(start, count);
            if free_ports.is_empty() {
                eprintln!("⚠ No free ports found starting from {}", start);
                std::process::exit(1);
            } else if count == 1 {
                println!("{}", free_ports[0]);
            } else {
                if cli.json {
                    println!("{}", serde_json::to_string_pretty(&free_ports)?);
                } else {
                    println!("Free ports starting from {}:", start);
                    for port in &free_ports {
                        println!("  {}", port);
                    }
                }
            }
        }

        Some(Commands::Conflicts { port }) => {
            let conflicts = port_utils::detect_conflicts(&config).await?;
            let filtered: Vec<_> = match port {
                Some(p) => conflicts.into_iter().filter(|c| c.port == p).collect(),
                None => conflicts,
            };

            if filtered.is_empty() {
                println!("✓ No port conflicts detected.");
            } else {
                for conflict in &filtered {
                    println!("Port {}/{}:", conflict.port, conflict.protocol);
                    for (i, proc) in conflict.processes.iter().enumerate() {
                        println!(
                            "  {}. PID {} - {} ({})",
                            i + 1,
                            proc.pid,
                            proc.name,
                            proc.command
                        );
                    }
                    println!("  Suggestion: {}", conflict.suggestion);
                    println!();
                }
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

fn select_entry(
    entries: &[PortEntry],
    port: u16,
    protocol: Option<Protocol>,
    pid: Option<u32>,
) -> Result<Option<&PortEntry>> {
    let matches: Vec<&PortEntry> = entries
        .iter()
        .filter(|entry| {
            entry.port == port
                && protocol.is_none_or(|protocol| entry.protocol == protocol)
                && pid.is_none_or(|pid| entry.pid == pid)
        })
        .collect();

    match matches.as_slice() {
        [] => Ok(None),
        [entry] => Ok(Some(*entry)),
        _ => Err(PortForgeError::ProcessError(format!(
            "Port {port} matches multiple listeners; rerun with --pid and/or --protocol"
        ))),
    }
}
