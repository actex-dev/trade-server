use sea_orm_migration::prelude::*;
use migration::{Migrator, MigratorTrait};

#[tokio::main]
async fn main() {
    let _ = dotenvy::dotenv();
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL not set");
    let db = sea_orm::Database::connect(&database_url)
        .await
        .expect("Failed to connect to database");

    Migrator::up(&db, None).await.expect("Migration failed");
}