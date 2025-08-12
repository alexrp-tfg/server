use std::{env, sync::Arc};

use anyhow::Context;
use tokio::net;
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};
use utoipa_swagger_ui::SwaggerUi;

use crate::{
    api::routes::{api_routes, combine_openapi},
    media::domain::{FileStorageService, MediaRepository, ThumbnailService},
    users::domain::{LoginTokenService, UserRepository},
};

// State that every handlers share (used for services)
#[derive(Clone)]
pub struct AppState {
    pub user_repository: Arc<dyn UserRepository>,
    pub login_token_service: Arc<dyn LoginTokenService>,
    pub media_repository: Arc<dyn MediaRepository>,
    pub storage_service: Arc<dyn FileStorageService>,
    pub thumbnail_service: Arc<dyn ThumbnailService>,
}

pub struct HttpServer {
    router: axum::Router,
    listener: net::TcpListener,
}

impl HttpServer {
    pub async fn new(
        user_repository: impl UserRepository + 'static,
        login_token_service: impl LoginTokenService + 'static,
        media_repository: impl MediaRepository + 'static,
        storage_service: impl FileStorageService + 'static,
        thumbnail_service: impl ThumbnailService + 'static,
    ) -> anyhow::Result<Self> {
        dotenvy::dotenv().context("Failed to load .env file")?;

        let state = AppState {
            user_repository: Arc::new(user_repository),
            login_token_service: Arc::new(login_token_service),
            media_repository: Arc::new(media_repository),
            storage_service: Arc::new(storage_service),
            thumbnail_service: Arc::new(thumbnail_service),
        };

        // Initialize tracing for the application
        tracing_subscriber::fmt::init();

        // CORS configuration
        let cors = CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(Any)
            .allow_headers(Any);

        let port = env::var("API_PORT")
            .context("Failed to read API_PORT from environment")?
            .parse::<u16>()
            .context("Failed to parse API_PORT as u16")?;

        // Create the Axum router with the API routes and Swagger UI
        let router = axum::Router::new()
            .layer(cors)
            .nest("/api", api_routes(state.clone()))
            .with_state(state)
            .merge(SwaggerUi::new("/doc").url("/api-docs/openapi.json", combine_openapi(&port)))
            .layer(TraceLayer::new_for_http());

        let listener = net::TcpListener::bind(format!("0.0.0.0:{}", port))
            .await
            .with_context(|| format!("Failed to start server on port {}", port))?;
        Ok(Self { router, listener })
    }

    /// Runs the HTTP server.
    pub async fn run(self) -> anyhow::Result<()> {
        println!(
            "Listening on port {}",
            self.listener
                .local_addr()
                .expect("Failed to get local address")
        );
        axum::serve(self.listener, self.router.into_make_service())
            .await
            .context("received error when running server")?;
        Ok(())
    }
}
