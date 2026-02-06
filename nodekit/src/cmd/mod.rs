use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "nodekit")]
#[command(about = "Manage Algorand nodes from the command line", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Data directory for the node
    #[arg(short, long, env = "ALGOD_DATA")]
    pub datadir: Option<String>,

    /// Disable setting incentive eligibility fees
    #[arg(short, long, default_value_t = false)]
    pub no_incentives: bool,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Bootstrap a new node
    Bootstrap,
    /// Debug the current node
    Debug,
    /// Install Algorand node
    Install,
    /// Start the Algorand node
    Start,
    /// Stop the Algorand node
    Stop,
    /// Uninstall Algorand node
    Uninstall,
    /// Upgrade the Algorand node
    Upgrade,
    /// Catchup subcommands
    Catchup {
        #[command(subcommand)]
        command: Option<CatchupCommands>,
    },
    /// Configure subcommands
    Configure {
        #[command(subcommand)]
        command: Option<ConfigureCommands>,
    },
    /// Telemetry subcommands
    Telemetry {
        #[command(subcommand)]
        command: Option<TelemetryCommands>,
    },
}

#[derive(Subcommand, Debug)]
pub enum CatchupCommands {
    /// Start fast catchup
    Start,
    /// Stop fast catchup
    Stop,
    /// Check catchup status
    Status,
}

#[derive(Subcommand, Debug)]
pub enum ConfigureCommands {
    /// Configure algod
    Algod,
    /// Configure service
    Service,
    /// Configure telemetry
    Telemetry,
}

#[derive(Subcommand, Debug)]
pub enum TelemetryCommands {
    /// Enable telemetry
    Enable,
    /// Disable telemetry
    Disable,
    /// Check telemetry status
    Status,
}
