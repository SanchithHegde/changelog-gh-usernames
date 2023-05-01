//! Interfaces to interact with the database.

use std::str::FromStr;

use anyhow::Context;
use sqlx::{sqlite::SqliteConnectOptions, SqlitePool};

/// A GitHub user entry stored in the database.
#[derive(Debug)]
pub(crate) struct GithubUser {
    /// The public email address associated with the GitHub user.
    pub(crate) email: String,

    /// The user's GitHub username.
    pub(crate) username: String,
}

impl GithubUser {
    /// Construct a new GitHub user entry.
    pub(crate) fn new(email: String, username: String) -> Self {
        Self { email, username }
    }
}

/// An interface to interact with [`GithubUser`] entries stored in the database.
#[derive(Debug)]
pub(crate) struct GithubUserRepository {
    /// The database pool used to interact with the database.
    pool: SqlitePool,
}

impl GithubUserRepository {
    /// Construct a [`GithubUserRepository`].
    pub(crate) fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Insert a [`GithubUser`] in the database.
    pub(crate) async fn insert(&self, user: GithubUser) -> anyhow::Result<()> {
        sqlx::query!(
            "INSERT INTO users (email, username) VALUES (?, ?)",
            user.email,
            user.username
        )
        .execute(&self.pool)
        .await
        .context("Failed to insert user in database")?;
        Ok(())
    }

    /// Retrieve a [`GithubUser`] from the database, with the specified email address.
    pub(crate) async fn get(&self, email: &str) -> anyhow::Result<Option<GithubUser>> {
        sqlx::query_as!(
            GithubUser,
            "SELECT email, username FROM users WHERE email = ?",
            email
        )
        .fetch_optional(&self.pool)
        .await
        .context("Failed to query users from database")
    }
}

/// Create a database pool used to interact with the database.
pub(crate) async fn create_pool(database_uri: impl AsRef<str>) -> anyhow::Result<SqlitePool> {
    let pool_options = SqliteConnectOptions::from_str(database_uri.as_ref())
        .context("Failed to parse database URI")?
        .create_if_missing(true);
    SqlitePool::connect_with(pool_options)
        .await
        .context("Failed to access database")
}

/// Run database migrations
pub(crate) async fn run_migrations(pool: &SqlitePool) -> anyhow::Result<()> {
    static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!();
    MIGRATOR
        .run(pool)
        .await
        .context("Failed to run database migrations")
}
