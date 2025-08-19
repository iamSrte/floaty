use diesel::prelude::*;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};

pub mod config;
pub mod models;
pub mod schema;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("./db/migrations");

/// Establishes a connection to the SQLite database.
pub fn establish_connection() -> SqliteConnection {
    let database_url = config::get_database_path().expect("Failed to get database path.");
    SqliteConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}

/// Runs pending migrations on the database connection.
pub fn run_migrations(
    connection: &mut SqliteConnection,
) -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
    connection.run_pending_migrations(MIGRATIONS)?;
    Ok(())
}
