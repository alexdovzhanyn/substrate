use std::sync::Arc;

use crate::info;
use axum::http::{HeaderValue, Method};
use rmcp::transport::streamable_http_server::{
  StreamableHttpServerConfig, StreamableHttpService, session::local::LocalSessionManager,
};
use tokio::sync::Mutex;
use tower_http::cors::CorsLayer;
use tower_http::services::{ServeDir, ServeFile};

use crate::core::SubstrateCore;
use crate::error::AppResult;
use crate::http::api;
use crate::mcp::service::MCPService;
use crate::util::Config;

pub async fn run(core: Arc<Mutex<SubstrateCore>>, config: Config) -> AppResult<()> {
  info!("[HTTP] Starting server...");

  let ct = tokio_util::sync::CancellationToken::new();

  let api_core = core.clone();

  let mcp_service = StreamableHttpService::new(
    move || Ok(MCPService::new(core.clone())),
    LocalSessionManager::default().into(),
    StreamableHttpServerConfig::default().with_cancellation_token(ct.child_token()),
  );

  let api_router = api::router(api_core);

  let web_service = ServeDir::new(get_colocated_path("web"))
    .not_found_service(ServeFile::new(get_colocated_path("web/index.html")));

  let bind_address = format!("127.0.0.1:{}", config.http.port);
  let tcp_listener = tokio::net::TcpListener::bind(&bind_address).await?;

  let cors = CorsLayer::new()
    .allow_origin("http://localhost:5173".parse::<HeaderValue>().unwrap())
    .allow_methods([
      Method::GET,
      Method::POST,
      Method::PUT,
      Method::DELETE,
      Method::OPTIONS,
    ])
    .allow_headers(tower_http::cors::Any);

  let app = axum::Router::new()
    .nest_service("/mcp", mcp_service)
    .nest("/api", api_router)
    .fallback_service(web_service)
    .layer(cors);

  info!("[HTTP] Server started. Listening on: {}", bind_address);

  axum::serve(tcp_listener, app)
    .with_graceful_shutdown(async move {
      tokio::signal::ctrl_c().await.unwrap();
      ct.cancel();
    })
    .await?;

  Ok(())
}

fn get_colocated_path(path: &str) -> std::path::PathBuf {
  std::env::current_exe()
    .unwrap()
    .parent()
    .unwrap()
    .join(path)
}
