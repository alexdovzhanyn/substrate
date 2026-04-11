use crate::info;
use rmcp::transport::streamable_http_server::{
  StreamableHttpServerConfig, StreamableHttpService, session::local::LocalSessionManager,
};

use tracing_subscriber::{
  layer::SubscriberExt,
  util::SubscriberInitExt,
  {self},
};

use crate::error::AppResult;
use crate::mcp::service::TesseractService;
use crate::state::AppState;
use crate::util::Config;

pub async fn run(state: AppState, config: Config) -> AppResult<()> {
  info!("[MCP] Starting server...");

  tracing_subscriber::registry()
    .with(
      tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| "info".to_string().into()),
    )
    .with(tracing_subscriber::fmt::layer())
    .init();

  let ct = tokio_util::sync::CancellationToken::new();

  let service = StreamableHttpService::new(
    move || Ok(TesseractService::new(state.clone())),
    LocalSessionManager::default().into(),
    StreamableHttpServerConfig::default().with_cancellation_token(ct.child_token()),
  );

  let bind_address = format!("127.0.0.1:{}", config.mcp.port);

  let router = axum::Router::new().nest_service("/mcp", service);
  let tcp_listener = tokio::net::TcpListener::bind(&bind_address).await?;

  info!("[MCP] Server started. Listening on: {}", bind_address);

  let _ = axum::serve(tcp_listener, router)
    .with_graceful_shutdown(async move {
      tokio::signal::ctrl_c().await.unwrap();
      ct.cancel();
    })
    .await;

  Ok(())
}
