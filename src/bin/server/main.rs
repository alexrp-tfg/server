use std::sync::Arc;

use diesel::{
    PgConnection,
    r2d2::{ConnectionManager, Pool},
};
use lib::{api::http_server::HttpServer, users::infrastructure::DieselUserRepository};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    // Establish a connection to the database and save as Arc so it can be shared among
    // multiple repositories
    let connection = Arc::new(establish_connection());

    let user_repository = DieselUserRepository::new(connection);

    let server = HttpServer::new(user_repository).await?;

    server.run().await
}

fn establish_connection() -> Pool<ConnectionManager<PgConnection>> {
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set in the environment variables");

    let manager = ConnectionManager::<PgConnection>::new(&database_url);

    Pool::builder().build(manager).expect(&format!(
        "Error creating connection pool for {}",
        &database_url
    ))
}
