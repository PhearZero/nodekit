#[cfg(target_arch = "wasm32")]
use nodekit::app::App;
#[cfg(target_arch = "wasm32")]
use nodekit::service;
#[cfg(target_arch = "wasm32")]
use ratzilla::DomBackend;
#[cfg(target_arch = "wasm32")]
use ratzilla::ratatui::Terminal;

#[cfg(target_arch = "wasm32")]
#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    color_eyre::install()?;

    let url = "http://localhost:8080".to_string();
    let token = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_string();

    let backend = DomBackend::new().map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;
    let terminal = Terminal::new(backend).map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;

    let app = App::new(&url, &token, None, false);
    
    // Start background services
    service::metrics::spawn_metrics_loop(
        app.events.get_sender(),
        url.clone(),
        token.clone(),
    );
    service::node::spawn_node_loop(
        app.events.get_sender(),
        app.client.clone(),
        None,
    );

    app.run(terminal).await?;

    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    eprintln!("The 'web' binary only runs when compiled to wasm32 (served via trunk).\nTry: \n  rustup target add wasm32-unknown-unknown\n  cargo install --locked trunk\n  trunk serve");
}
