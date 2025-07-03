use std::env;

use anyhow::Context;
use tokio::net;
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};
use utoipa_swagger_ui::SwaggerUi;

use crate::api::routes::{api_routes, combine_openapi};

// State that every handlers share (used for services)
#[derive(Debug, Clone)]
pub struct AppState {}

pub struct HttpServer {
    router: axum::Router,
    listener: net::TcpListener,
}

impl HttpServer {
    pub async fn new() -> anyhow::Result<Self> {
        dotenvy::dotenv().context("Failed to load .env file")?;

        let db_url =
            env::var("DATABASE_URL").context("Failed to read DATABASE_URL from environment")?;

        // Connect to the database
        let connection = sqlx::PgPool::connect(&db_url)
            .await
            .context("Failed to connect to the database")?;

        // Run database migrations
        sqlx::migrate!("./migrations")
            .run(&mut connection.acquire().await?)
            .await
            .context("Failed to run database migrations")?;

        let state = AppState {};

        // Initialize tracing for the application
        tracing_subscriber::fmt::init();

        // CORS configuration
        let cors = CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(Any)
            .allow_headers(Any);

        // Create the Axum router with the API routes and Swagger UI
        let router = axum::Router::new()
            .layer(cors)
            .nest("/api", api_routes())
            .with_state(state)
            .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", combine_openapi()))
            .layer(TraceLayer::new_for_http());

        let port = env::var("API_PORT")
            .context("Failed to read API_PORT from environment")?
            .parse::<u16>()
            .context("Failed to parse API_PORT as u16")?;

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
