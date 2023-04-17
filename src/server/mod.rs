//! Main application server
use std::sync::Arc;

use actix_cors::Cors;
use actix_web::{dev, http::StatusCode, middleware::ErrorHandlers, web, App, HttpServer};
use cadence::StatsdClient;

use crate::server::dockerflow::configure;
use crate::web::middleware::sentry::SentryWrapper;
use crate::{error::HandlerError, metrics, settings::Settings};

mod dockerflow;

/// This is the global HTTP state object that will be made available to all
/// HTTP API calls.
#[derive(Clone, Debug)]
pub struct ServerState {
    /// Metric reporting
    pub metrics: Arc<StatsdClient>,
    pub port: u16,
}

pub struct Server;

#[macro_export]
macro_rules! build_app {
    ($state: expr) => {
        // If you want to customize how sentry handles data or reports errors, you're
        // going to need to do some leg work here.
        App::new()
            .app_data($state)
            // Middleware is applied LIFO
            // These will wrap all outbound responses with matching status codes.
            .wrap(ErrorHandlers::new().handler(StatusCode::NOT_FOUND, HandlerError::render_404))
            // These are our wrappers
            .wrap(SentryWrapper::default())
            // or use the default sentry wrapper
            //  .wrap(sentry_actix::Sentry::builder().capture_server_errors(true).finish())
            // Followed by the "official middleware" so they run first.
            // actix is getting increasingly tighter about CORS headers. Our server is
            // not a huge risk but does deliver XHR JSON content.
            // For now, let's be permissive and use NGINX (the wrapping server)
            // for finer grained specification.
            .wrap(Cors::permissive())
            .service(web::scope("").configure(configure))
    };
}

impl Server {
    pub async fn with_settings(settings: Settings) -> Result<dev::Server, HandlerError> {
        let state = ServerState {
            metrics: Arc::new(metrics::metrics_from_opts(&settings)?),
            port: settings.port,
        };
        let mut server = HttpServer::new(move || build_app!(state.clone()));
        if let Some(keep_alive) = settings.actix_keep_alive {
            server = server.keep_alive(std::time::Duration::from_secs(keep_alive));
        }
        let server = server
            .bind((settings.host, settings.port))
            .expect("Could not get Server in Server::with_settings")
            .run();
        Ok(server)
    }
}
