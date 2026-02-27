mod service;

use clap::Parser;
use crate::app::App;
use crate::cmd::Cli;

pub mod app;
pub mod cmd;
pub mod event;
pub mod ui;

#[cfg_attr(not(target_arch = "wasm32"), tokio::main)]
#[cfg_attr(target_arch = "wasm32", tokio::main(flavor = "current_thread"))]
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
            crate::cmd::Commands::Web { port } => {
                #[cfg(not(target_arch = "wasm32"))]
                {
                    println!(
                        "Web mode is only available when targeting wasm32.\n\nTo run the web UI: \n  1) rustup target add wasm32-unknown-unknown\n  2) cargo install --locked trunk\n  3) cd nodekit && trunk serve\nThen open http://localhost:{} in your browser.",
                        port
                    );
                    return Ok(());
                }
                #[cfg(target_arch = "wasm32")]
                {
                    // The wasm/web entrypoint is defined in src/bin/web.rs
                    // This path should not be hit under wasm target for this binary.
                    return Ok(());
                }
            }
        }
        return Ok(());
    }

    // If no command is provided, run the TUI
    #[cfg(not(target_arch = "wasm32"))]
    {
        let terminal = ratatui::init();
        let app = App::new(&cli.url, &cli.token, cli.data_dir.as_deref(), cli.no_incentives);
        
        // Start background services
        service::metrics::spawn_metrics_loop(
            app.events.get_sender(),
            cli.url.clone(),
            cli.token.clone(),
        );
        service::node::spawn_node_loop(
            app.events.get_sender(),
            app.client.clone(),
            None,
        );

        let result = app.run(terminal).await;
        ratatui::restore();
        result
    }
    #[cfg(target_arch = "wasm32")]
    {
        Ok(())
    }
}
