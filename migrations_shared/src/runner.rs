use crate::error::*;
use bb8_postgres::{
    bb8::{Pool, PooledConnection},
    tokio_postgres::Transaction,
    PostgresConnectionManager,
};
use std::collections::HashSet;
use tokio_postgres::NoTls;

/// Create table statement for the `__schema_migrations` used by the postgresql
pub const CREATE_MIGRATIONS_TABLE: &str = include_str!("setup_migration_table.sql");

pub type DbConnectionPool = Pool<PostgresConnectionManager<NoTls>>;
pub type DbPooledConnection<'a> = PooledConnection<'a, PostgresConnectionManager<NoTls>>;
pub type DbTransaction<'a> = Transaction<'a>;

pub struct MigrationsRunner {
    pool: DbConnectionPool,
}

impl MigrationsRunner {
    pub fn from_pool(pool: DbConnectionPool) -> MigrationsRunner {
        MigrationsRunner { pool }
    }

    pub async fn get_pooled_conn(&self) -> Result<DbPooledConnection<'_>, Error> {
        let conn = self.pool.get().await?;
        Ok(conn)
    }

    pub async fn setup_database(&self) -> Result<(), Error> {
        let conn = self.pool.get().await?;
        conn.execute(CREATE_MIGRATIONS_TABLE, &[]).await?;
        Ok(())
    }

    pub async fn previously_run_migration_versions(&self) -> Result<HashSet<String>, Error> {
        let conn = self.pool.get().await?;
        let mut migrations = HashSet::new();
        let query = "SELECT version FROM __schema_migrations";
        for row in &conn.query(query, &[]).await? {
            migrations.insert(row.try_get(0)?);
        }
        Ok(migrations)
    }

    pub async fn latest_run_migration_version(&self) -> Result<Option<String>, Error> {
        let conn = self.pool.get().await?;
        let query = "SELECT MAX(version) FROM __schema_migrations";
        if let Some(row) = &conn.query_opt(query, &[]).await? {
            return Ok(Some(row.try_get(0)?));
        }
        Ok(None)
    }

    pub async fn insert_new_migration(&self, transaction: &DbTransaction<'_>, ver: &str) -> Result<(), Error> {
        let query = "INSERT INTO __schema_migrations VALUES($1)";
        transaction.execute(query, &[&ver]).await?;
        Ok(())
    }

    pub async fn delete_migration(&self, transaction: &DbTransaction<'_>, ver: &str) -> Result<(), Error> {
        let query = "DELETE FROM __schema_migrations WHERE version=$1";
        transaction.execute(query, &[&ver]).await?;
        Ok(())
    }
}
