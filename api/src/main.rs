use actix_web::{middleware, web, App, HttpMessage, HttpServer};
use std::sync::Arc;
use tokio::signal;

use api::config::global_state::AppState;
use api::config::settings::Config;
use api::logging::formatter::init_tracing;
use api::middleware::request_context::RequestContext;
use api::routes;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Load environment variables
    dotenv::dotenv().ok();

    // Load configuration
    let config = Config::from_env();

    // Initialize logging
    init_tracing();

    tracing::info!(
        host = %config.server_host,
        port = %config.server_port,
        env = %config.env,
        "Starting TaskFlow API server"
    );

    // Initialize application state
    let state = match AppState::new(config.clone()).await {
        Ok(s) => Arc::new(s),
        Err(e) => {
            tracing::error!("Failed to initialize application state: {}", e);
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to initialize: {}", e),
            ));
        }
    };

    let host = config.server_host.clone();
    let port = config.server_port;

    let server = HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new((*state.clone()).clone()))
            // Request context middleware — injects requestId into every request
            .wrap(middleware::from_fn(|req: actix_web::dev::ServiceRequest, next: actix_web::middleware::Next<actix_web::body::BoxBody>| async move {
                let ctx = RequestContext::new();
                req.extensions_mut().insert(ctx);
                next.call(req).await
            }))
            .wrap(middleware::Logger::default())
            .wrap(
                middleware::DefaultHeaders::new()
                    .add(("Content-Type", "application/json")),
            )
            .configure(routes::configure)
    })
    .bind((host.as_str(), port))?
    .run();

    tracing::info!("Server running at http://{}:{}", host, port);

    // Graceful shutdown
    let server_handle = server.handle();
    tokio::spawn(async move {
        let ctrl_c = async {
            signal::ctrl_c()
                .await
                .expect("Failed to install Ctrl+C handler");
        };

        #[cfg(unix)]
        let terminate = async {
            signal::unix::signal(signal::unix::SignalKind::terminate())
                .expect("Failed to install signal handler")
                .recv()
                .await;
        };

        #[cfg(not(unix))]
        let terminate = std::future::pending::<()>();

        tokio::select! {
            _ = ctrl_c => {},
            _ = terminate => {},
        }

        tracing::info!("Shutdown signal received, stopping server gracefully...");
        server_handle.stop(true).await;
    });

    server.await
}
