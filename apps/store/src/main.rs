//! Bullpen store: conversations, memories, the profile graph and confirmations on Postgres.

mod core;
mod db;
mod routes;

use std::time::Duration;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    if let Err(e) = dotenvy::dotenv()
        && !e.not_found()
    {
        return Err(anyhow::anyhow!(".env: {e}"));
    }
    let cfg = core::config::Config::load()?;
    let _guard = core::telemetry::init(&cfg.telemetry, &cfg.app.env, &cfg.app.log_level)?;
    // Config::validate guarantees the token and the database url.
    let (Some(url), Some(token)) = (cfg.store.database_url.as_ref(), cfg.store.token.clone())
    else {
        anyhow::bail!("set BULLPEN_STORE__DATABASE_URL and BULLPEN_STORE__TOKEN");
    };
    let db = db::Db::connect(
        url,
        cfg.store.max_connections,
        Duration::from_secs(cfg.store.acquire_timeout_secs),
    )
    .await?;
    db.migrate().await?;
    let app = routes::router(
        routes::AppState::new(db, token, &cfg.store),
        cfg.store.max_body_bytes,
    );
    let listener = tokio::net::TcpListener::bind(&cfg.store.bind).await?;
    tracing::info!(env = %cfg.app.env, bind = %cfg.store.bind, "store listening");
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown())
        .await?;
    Ok(())
}

/// Resolves on ctrl-c or SIGTERM.
async fn shutdown() {
    let ctrl_c = async {
        if let Err(e) = tokio::signal::ctrl_c().await {
            tracing::warn!(error = %e, "ctrl-c handler failed");
        }
    };
    #[cfg(unix)]
    let term = async {
        match tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()) {
            Ok(mut signal) => {
                signal.recv().await;
            }
            Err(e) => tracing::warn!(error = %e, "SIGTERM handler failed"),
        }
    };
    #[cfg(not(unix))]
    let term = std::future::pending::<()>();
    tokio::select! {
        () = ctrl_c => {},
        () = term => {},
    }
    tracing::info!("store shutting down");
}
