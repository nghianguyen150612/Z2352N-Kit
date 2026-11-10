mod device;
mod research;

use clap::{Parser, Subcommand};
use serde::Serialize;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Parser)]
#[command(
    name = "z2352n",
    version,
    about = "Nubia Neo 2 5G (Z2352N) research toolkit"
)]
#[command(
    long_about = "P001: read-only diagnostics. This build never writes device partitions, unlocks, resets FRP, or flashes images."
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Check host tools, without changing any device state
    Doctor {
        #[arg(long)]
        json: bool,
    },
    /// Detect one connected Android/fastboot device (read-only)
    Probe {
        #[arg(long)]
        json: bool,
    },
    /// Assess bootloader status from reported Android properties
    Unlock {
        #[command(subcommand)]
        command: UnlockCommands,
    },
    /// Ownership recovery information; does not bypass FRP
    Frp {
        #[command(subcommand)]
        command: FrpCommands,
    },
    /// Read-only offline research tools
    Research {
        #[command(subcommand)]
        command: ResearchCommands,
    },
    /// Produce a redacted machine-readable host/device report
    Report {
        #[arg(long)]
        json: bool,
    },
    /// Run the safe first-connection workflow
    Wizard {
        #[arg(long)]
        json: bool,
    },
}

#[derive(Subcommand)]
enum UnlockCommands {
    /// Check reported lock state; DOES NOT perform an unlock
    Status {
        #[arg(long)]
        json: bool,
    },
}

#[derive(Subcommand)]
enum FrpCommands {
    /// Show official account/ownership recovery options
    Recovery,
}

#[derive(Subcommand)]
enum ResearchCommands {
    /// SHA-256 of a local file, streamed without modifying it
    Hash {
        path: PathBuf,
        #[arg(long)]
        json: bool,
    },
}

#[derive(Serialize)]
struct Report {
    schema_version: u32,
    generated_at_unix: u64,
    host: device::HostTools,
    device: device::DeviceProbe,
    safety: &'static str,
}

fn main() {
    let cli = Cli::parse();
    if let Err(message) = execute(cli.command) {
        eprintln!("Error: {message}");
        std::process::exit(1);
    }
}

fn execute(command: Commands) -> Result<(), String> {
    let runner = device::SystemRunner;
    match command {
        Commands::Doctor { json } => {
            let host = device::HostTools::detect();
            if json {
                print_json(&host)
            } else {
                println!("Host diagnostics (read-only)");
                println!("ADB:       {}", present(host.adb));
                println!("Fastboot:  {}", present(host.fastboot));
                println!("lsusb:     {}", present(host.lsusb));
                println!("Python 3:  {}", present(host.python3));
                if !host.adb {
                    println!("Hint: install Android platform-tools for Android detection.");
                }
                Ok(())
            }
        }
        Commands::Probe { json } => {
            let probe = device::probe(&runner);
            if json {
                print_json(&probe)
            } else {
                print_probe(&probe);
                Ok(())
            }
        }
        Commands::Unlock {
            command: UnlockCommands::Status { json },
        } => {
            let probe = device::probe(&runner);
            if json {
                print_json(&probe)
            } else {
                print_probe(&probe);
                println!(
                    "Unlock is NOT performed. Reported properties alone do not prove an OEM unlock."
                );
                Ok(())
            }
        }
        Commands::Frp {
            command: FrpCommands::Recovery,
        } => {
            println!("FRP / ownership recovery");
            println!("Use the Google account previously synced to the device.");
            println!("Google account recovery: https://accounts.google.com/signin/recovery");
            println!("For a second-hand phone, ask the previous owner to remove their account.");
            println!("For service options, contact the manufacturer with proof of ownership.");
            println!("This toolkit does not bypass or delete FRP protection.");
            Ok(())
        }
        Commands::Research {
            command: ResearchCommands::Hash { path, json },
        } => {
            let result = research::hash_file(&path)?;
            if json {
                print_json(&result)
            } else {
                println!(
                    "{}  {}  ({} bytes)",
                    result.sha256, result.filename, result.bytes
                );
                Ok(())
            }
        }
        Commands::Report { json } | Commands::Wizard { json } => {
            let report = Report {
                schema_version: 1,
                generated_at_unix: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .map_err(|e| e.to_string())?
                    .as_secs(),
                host: device::HostTools::detect(),
                device: device::probe(&runner),
                safety: "read_only_no_flash_no_unlock_no_frp_bypass",
            };
            if json {
                print_json(&report)
            } else {
                println!("Z2352N-Kit P001 — safe connection workflow");
                println!("ADB: {}", present(report.host.adb));
                println!("Fastboot: {}", present(report.host.fastboot));
                print_probe(&report.device);
                println!("Next: save a redacted report with: z2352n report --json");
                Ok(())
            }
        }
    }
}

fn print_probe(probe: &device::DeviceProbe) {
    println!("Connection: {}", probe.connection.as_str());
    println!("Z2352N identity: {}", probe.identity.as_str());
    if let Some(model) = &probe.model {
        println!("Model (reported): {model}");
    }
    if let Some(board) = &probe.board {
        println!("Board (reported): {board}");
    }
    if let Some(version) = &probe.android_version {
        println!("Android: {version}");
    }
    println!("Bootloader: {}", probe.lock.assessment.as_str());
    for note in &probe.notes {
        println!("Note: {note}");
    }
}

fn present(yes: bool) -> &'static str {
    if yes { "found" } else { "missing" }
}

fn print_json<T: Serialize>(value: &T) -> Result<(), String> {
    println!(
        "{}",
        serde_json::to_string_pretty(value).map_err(|e| e.to_string())?
    );
    Ok(())
}
