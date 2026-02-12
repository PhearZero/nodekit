use clap::Parser;
use crate::app::App;
use crate::cmd::Cli;

pub mod app;
pub mod cmd;
pub mod event;
pub mod ui;

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    let cli = Cli::parse();

    if let Some(command) = cli.command {
        match command {
            crate::cmd::Commands::Debug => println!("Debugging..."),
            crate::cmd::Commands::Start => println!("Starting..."),
            crate::cmd::Commands::Stop => println!("Stopping..."),
            crate::cmd::Commands::Upgrade => println!("Upgrading..."),
            crate::cmd::Commands::Catchup { command } => println!("Catchup: {:?}", command),
            crate::cmd::Commands::Configure { command } => println!("Configure: {:?}", command),
            crate::cmd::Commands::Telemetry { command } => println!("Telemetry: {:?}", command),
        }
        return Ok(());
    }

    // If no command is provided, run the TUI
    let terminal = ratatui::init();
    let result = App::new(&cli.url, &cli.token, cli.data_dir.as_deref(), cli.no_incentives).run(terminal).await;
    ratatui::restore();
    result
}
