use diesel::prelude::*;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};

use deadpool_diesel::sqlite::{Manager, Pool};
use deadpool_diesel::Runtime::Tokio1;

pub mod config;
pub mod models;
pub mod schema;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("./db/migrations");

pub type DbPool = Pool;

/// Creates a connection pool for the SQLite database.
pub fn create_pool() -> DbPool {
    let database_url = config::get_database_path().expect("Failed to get database path.");
    let manager = Manager::new(database_url, Tokio1);

    Pool::builder(manager)
        .max_size(15)
        .build()
        .expect("Failed to create database pool")
}

/// Runs pending migrations on the database connection.
pub fn run_migrations(
    connection: &mut SqliteConnection,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    connection.run_pending_migrations(MIGRATIONS)?;
    Ok(())
}
