use crate::app::App;
use clap::Parser;

pub mod app;
pub mod event;
pub mod ui;
pub mod config;
pub mod systemd;
pub mod upgrade;
pub mod server;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Start the daemon service
    #[arg(short, long)]
    daemon: bool,

    /// TLS certificate file path
    #[arg(long)]
    cert: Option<std::path::PathBuf>,

    /// TLS key file path
    #[arg(long)]
    key: Option<std::path::PathBuf>,
}

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let args = Args::parse();

    if args.daemon {
        tracing_subscriber::fmt::init();
        let data_dir = config::get_data_dir().map_err(|e| color_eyre::eyre::eyre!(e))?;
        let algod_config = config::load_algorand_config(&data_dir).map_err(|e| color_eyre::eyre::eyre!(e))?;
        server::run_server(algod_config, args.cert, args.key).await.map_err(|e| color_eyre::eyre::eyre!(e))?;
    } else {
        let terminal = ratatui::init();
        let result = App::new().run(terminal).await;
        ratatui::restore();
        result?;
    }
    
    Ok(())
}
