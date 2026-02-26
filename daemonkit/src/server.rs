use axum::{
    extract::{Path, State},
    http::{Request, Response, StatusCode},
    middleware::{self, Next},
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use std::sync::Arc;
use crate::config::AlgorandConfig;
use crate::systemd;
use crate::upgrade;
use reqwest::Client;
use tower_http::trace::TraceLayer;
use anyhow::Result;
use std::path::PathBuf;
use axum_server::tls_rustls::RustlsConfig;

struct AppState {
    config: AlgorandConfig,
    client: Client,
}

pub async fn run_server(config: AlgorandConfig, cert: Option<PathBuf>, key: Option<PathBuf>) -> Result<()> {
    let state = Arc::new(AppState {
        config,
        client: Client::new(),
    });

    let app = Router::new()
        .route("/v2/status/wait-for-block-after/:round", get(wait_for_block))
        .route("/start", post(start_node))
        .route("/stop", post(stop_node))
        .route("/upgrade", post(upgrade_node))
        .fallback(proxy_handler)
        .layer(middleware::from_fn_with_state(state.clone(), auth_middleware))
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let addr = "0.0.0.0:8080".parse()?;
    
    match (cert, key) {
        (Some(c), Some(k)) => {
            tracing::info!("Listening on {} with TLS", addr);
            let tls_config = RustlsConfig::from_pem_file(c, k).await?;
            axum_server::bind_rustls(addr, tls_config)
                .serve(app.into_make_service())
                .await?;
        }
        _ => {
            tracing::info!("Listening on {} (HTTP)", addr);
            axum_server::bind(addr)
                .serve(app.into_make_service())
                .await?;
        }
    }

    Ok(())
}

async fn auth_middleware(
    State(state): State<Arc<AppState>>,
    req: Request<axum::body::Body>,
    next: Next,
) -> Result<Response<axum::body::Body>, StatusCode> {
    let auth_header = req.headers().get("X-Algod-API-Token");
    
    match auth_header {
        Some(val) if val == &state.config.token => Ok(next.run(req).await),
        _ => Err(StatusCode::UNAUTHORIZED),
    }
}

async fn wait_for_block(
    State(state): State<Arc<AppState>>,
    Path(round): Path<u64>,
    req: Request<axum::body::Body>,
) -> impl IntoResponse {
    proxy_request(state, format!("/v2/status/wait-for-block-after/{}", round), req).await
}

async fn start_node() -> impl IntoResponse {
    match systemd::start_service().await {
        Ok(_) => (StatusCode::OK, "Service started").into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to start service: {}", e)).into_response(),
    }
}

async fn stop_node() -> impl IntoResponse {
    match systemd::stop_service().await {
        Ok(_) => (StatusCode::OK, "Service stopped").into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to stop service: {}", e)).into_response(),
    }
}

async fn upgrade_node() -> impl IntoResponse {
    match upgrade::upgrade_algorand() {
        Ok(_) => (StatusCode::OK, "Upgrade started").into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Upgrade failed: {}", e)).into_response(),
    }
}

async fn proxy_handler(
    State(state): State<Arc<AppState>>,
    req: Request<axum::body::Body>,
) -> impl IntoResponse {
    let path = req.uri().path().to_string();
    proxy_request(state, path, req).await
}

async fn proxy_request(
    state: Arc<AppState>,
    path: String,
    req: Request<axum::body::Body>,
) -> Response<axum::body::Body> {
    let target_url = format!("{}{}", state.config.endpoint, path);
    let mut proxy_req = state.client.request(req.method().clone(), &target_url);

    for (name, value) in req.headers().iter() {
        if name != "host" {
            proxy_req = proxy_req.header(name, value);
        }
    }
    
    // Add the token for the internal request to algod
    proxy_req = proxy_req.header("X-Algod-API-Token", &state.config.token);

    let (_parts, body) = req.into_parts();
    let body_bytes = match axum::body::to_bytes(body, usize::MAX).await {
        Ok(b) => b,
        Err(_) => return StatusCode::BAD_REQUEST.into_response(),
    };
    
    let res = match proxy_req.body(body_bytes).send().await {
        Ok(r) => r,
        Err(e) => {
            tracing::error!("Proxy error: {}", e);
            return StatusCode::BAD_GATEWAY.into_response();
        }
    };

    let mut builder = Response::builder().status(res.status());
    for (name, value) in res.headers().iter() {
        builder = builder.header(name, value);
    }

    let res_body_bytes = match res.bytes().await {
        Ok(b) => b,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };

    builder.body(axum::body::Body::from(res_body_bytes)).unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())
}
