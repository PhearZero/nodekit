use clap::Parser;
use nodekit::app::App;
use nodekit::cmd::Cli;
use nodekit::service;

#[cfg_attr(not(any(target_arch = "wasm32", target_arch = "xtensa", target_arch = "riscv32")), tokio::main)]
#[cfg_attr(target_arch = "wasm32", tokio::main(flavor = "current_thread"))]
async fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    let cli = Cli::parse();

    #[cfg(not(any(target_arch = "wasm32", target_arch = "xtensa", target_arch = "riscv32", feature = "simulator", feature = "mock-simulator")))]
    if let Some(command) = cli.command {
        match command {
            nodekit::cmd::Commands::Debug => println!("Debugging..."),
            nodekit::cmd::Commands::Start => println!("Starting..."),
            nodekit::cmd::Commands::Stop => println!("Stopping..."),
            nodekit::cmd::Commands::Upgrade => println!("Upgrading..."),
            nodekit::cmd::Commands::Catchup { command } => println!("Catchup: {:?}", command),
            nodekit::cmd::Commands::Configure { command } => println!("Configure: {:?}", command),
            nodekit::cmd::Commands::Telemetry { command } => println!("Telemetry: {:?}", command),
            nodekit::cmd::Commands::Web { port } => {
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
    #[cfg(not(any(target_arch = "wasm32", target_arch = "xtensa", target_arch = "riscv32")))]
    {
        #[cfg(feature = "mock-simulator")]
        {
            use mousefood::{EmbeddedBackend, EmbeddedBackendConfig};
            use embedded_graphics::pixelcolor::Rgb888;
            use embedded_graphics::mock_display::MockDisplay;
            use ratatui::Terminal;

            let mut display = MockDisplay::<Rgb888>::new();
            display.set_allow_out_of_bounds_drawing(true);

            let app = App::new(&cli.url, &cli.token, cli.data_dir.as_deref(), cli.no_incentives);
            
            // Start background services (optional for mock, but good for testing)
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

            let terminal = Terminal::new(EmbeddedBackend::new(&mut display, EmbeddedBackendConfig::default()))?;
            
            // For mock simulator on host, we can just run the embedded loop
            // but we need to make sure we don't block forever or we can at least see something
            println!("Running Mock Simulator (800x480). Close with Ctrl-C.");
            app.run(terminal).await
        }
        #[cfg(feature = "simulator")]
        {
            use embedded_graphics::pixelcolor::Rgb888;
            use embedded_graphics::prelude::*;
            use embedded_graphics_simulator::{
                SimulatorDisplay, Window, OutputSettingsBuilder,
            };
            use mousefood::{EmbeddedBackend, EmbeddedBackendConfig};
            use ratatui::Terminal;

            let mut display = SimulatorDisplay::<Rgb888>::new(Size::new(800, 480));
            let output_settings = OutputSettingsBuilder::new()
                .scale(1)
                .build();
            let window = Window::new("NodeKit Simulator", &output_settings);

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

            let terminal = Terminal::new(EmbeddedBackend::new(&mut display, EmbeddedBackendConfig::default()))?;
            app.run_simulator(terminal, window).await
        }
        #[cfg(not(any(feature = "simulator", feature = "mock-simulator")))]
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
    }
    #[cfg(target_arch = "wasm32")]
    {
        Ok(())
    }
}
