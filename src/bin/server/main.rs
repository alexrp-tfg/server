use std::sync::Arc;

use diesel::{
    PgConnection,
    r2d2::{ConnectionManager, Pool},
};
use lib::{
    api::http_server::HttpServer,
    media::{
        domain::ImageThumbnailService,
        infrastructure::{DieselMediaRepository, MinioStorageService},
    },
    users::{
        application::create_user::create_user_command_handler,
        infrastructure::{
            DieselUserRepository,
            jwt_token_service::{JwtTokenConfig, JwtTokenService},
        },
    },
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    // Check for CLI arguments
    let args: Vec<String> = std::env::args().collect();

    if args.len() > 1 {
        match args[1].as_str() {
            "create-admin" => {
                return create_admin_user().await;
            }
            _ => {
                eprintln!("Unknown command: {}", args[1]);
                eprintln!("Available commands: create-admin");
                std::process::exit(1);
            }
        }
    }

    // Establish a connection to the database and save as Arc so it can be shared among
    // multiple repositories
    let connection_pool = Arc::new(establish_connection());

    // Services
    let user_repository = DieselUserRepository::new(connection_pool.clone());
    let login_token_service = JwtTokenService::new(JwtTokenConfig::new());

    // Media services
    let media_repository = DieselMediaRepository::new((*connection_pool).clone());
    let storage_service = create_storage_service().await?;
    let thumbnail_service = ImageThumbnailService::new(
        DieselMediaRepository::new((*connection_pool).clone()),
        create_storage_service().await?,
    );

    let server = HttpServer::new(
        user_repository,
        login_token_service,
        media_repository,
        storage_service,
        thumbnail_service,
    )
    .await?;

    server.run().await
}

async fn create_admin_user() -> anyhow::Result<()> {
    println!("Creating admin user...");

    // Get admin credentials from environment variables
    let admin_username = std::env::var("ADMIN_USERNAME").unwrap_or_else(|_| "admin".to_string());
    let admin_password = std::env::var("ADMIN_PASSWORD").unwrap_or_else(|_| "admin".to_string());

    // Establish connection
    let connection_pool = Arc::new(establish_connection());
    let user_repository = DieselUserRepository::new(connection_pool);

    // Create admin user
    match create_user_command_handler(
        lib::users::application::create_user::CreateUserCommand {
            username: admin_username.clone(),
            password: admin_password,
            role: Some(lib::users::domain::Role::Admin),
        },
        &user_repository,
    )
    .await
    {
        Ok(_) => {
            println!(
                "Admin user created successfully with username: {}",
                admin_username
            );
            println!("Please change the default password on first login!");
        }
        Err(e) => {
            eprintln!("Failed to create admin user: {}", e);
            std::process::exit(1);
        }
    }

    Ok(())
}

fn establish_connection() -> Pool<ConnectionManager<PgConnection>> {
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set in the environment variables");

    let manager = ConnectionManager::<PgConnection>::new(&database_url);

    Pool::builder()
        .build(manager)
        .unwrap_or_else(|_| panic!("Error creating connection pool for {}", &database_url))
}

async fn create_storage_service() -> anyhow::Result<MinioStorageService> {
    let minio_endpoint =
        std::env::var("MINIO_ENDPOINT").unwrap_or_else(|_| "http://localhost:9000".to_string());
    let minio_access_key =
        std::env::var("MINIO_ACCESS_KEY").unwrap_or_else(|_| "minioadmin".to_string());
    let minio_secret_key =
        std::env::var("MINIO_SECRET_KEY").unwrap_or_else(|_| "minioadmin".to_string());
    let minio_bucket = std::env::var("MINIO_BUCKET").unwrap_or_else(|_| "media-files".to_string());

    MinioStorageService::new(
        minio_endpoint,
        minio_access_key,
        minio_secret_key,
        minio_bucket,
    )
    .await
    .map_err(|e| anyhow::anyhow!("Failed to create storage service: {}", e))
}
