pub mod error;
pub mod migration;
pub mod runner;

pub use async_trait::async_trait;
use error::*;
use migration::*;
use runner::*;
use std::env;
use std::fs::DirEntry;
use std::io::Write;
use std::path::{Path, PathBuf};

/// Run all pending migrations in the given list. Apps should likely be calling
pub async fn run_migrations<T>(
    pool: DbConnectionPool,
    migrations: Vec<T>,
    output: &mut (dyn Write + Send + Sync),
) -> Result<(), Error>
where
    T: Migration + Send + Sync,
{
    let runner = MigrationsRunner::from_pool(pool.clone());
    runner.setup_database().await?;
    let already_run = runner.previously_run_migration_versions().await?;
    let mut pending_migrations: Vec<_> = migrations
        .into_iter()
        .filter(|m| !already_run.contains(&m.version().to_string()))
        .collect();

    pending_migrations.sort_by(|a, b| a.version().cmp(b.version()));
    for migration in pending_migrations {
        run_migration(&runner, &migration, output).await?;
    }
    Ok(())
}

pub async fn run_migration(
    runner: &MigrationsRunner,
    migration: &(dyn Migration + Send + Sync),
    output: &mut (dyn Write + Send + Sync),
) -> Result<(), Error> {
    let mut conn = runner.get_pooled_conn().await?;
    let transaction = conn.transaction().await?;
    if migration.version() != "00000000000000" {
        writeln!(output, "Running migration {}", name(&migration))?;
    }
    if let Err(e) = migration.run(&transaction).await {
        writeln!(output, "Executing migration script {}", file_name(&migration, "up.sql"))?;
        return Err(e);
    }
    runner.insert_new_migration(&transaction, migration.version()).await?;
    transaction.commit().await?;
    Ok(())
}

pub async fn revert_migration(
    runner: &MigrationsRunner,
    migration: &(dyn Migration + Send + Sync),
    output: &mut (dyn Write + Send + Sync),
) -> Result<(), Error> {
    let mut conn = runner.get_pooled_conn().await?;
    let transaction = conn.transaction().await?;
    writeln!(output, "Rolling back migration {}", name(&migration))?;
    if let Err(e) = migration.revert(&transaction).await {
        writeln!(
            output,
            "Executing migration script {}",
            file_name(&migration, "down.sql")
        )?;
        return Err(e);
    }
    runner.delete_migration(&transaction, migration.version()).await?;
    transaction.commit().await?;
    Ok(())
}

/// Returns the directory containing migrations. Will look at for
/// $PWD/migrations. If it is not found, it will search the parents of the
/// current directory, until it reaches the root directory.  Returns
/// `MigrationError::MigrationDirectoryNotFound` if no directory is found.
pub fn find_migrations_directory() -> Result<PathBuf, Error> {
    search_for_migrations_directory(&env::current_dir()?)
}

/// Searches for the migrations directory relative to the given path. See
/// `find_migrations_directory` for more details.
pub fn search_for_migrations_directory(path: &Path) -> Result<PathBuf, Error> {
    let migration_path = path.join("migrations");
    if migration_path.is_dir() {
        Ok(migration_path)
    } else {
        path.parent()
            .map(|p| search_for_migrations_directory(p))
            .unwrap_or_else(|| Err(Error::MigrationDirectoryNotFound(path.into())))
            .map_err(|_| Error::MigrationDirectoryNotFound(path.into()))
    }
}

#[doc(hidden)]
pub fn migration_paths_in_directory(path: &Path) -> Result<Vec<DirEntry>, Error> {
    path.read_dir()?
        .filter_map(|entry| {
            let entry = match entry {
                Ok(e) => e,
                Err(e) => return Some(Err(e.into())),
            };
            if entry.file_name().to_string_lossy().starts_with('.') {
                None
            } else {
                Some(Ok(entry))
            }
        })
        .collect()
}

pub fn version_from_path(path: &Path) -> Result<String, Error> {
    path.file_name()
        .unwrap_or_else(|| panic!("Can't get file name from path `{:?}`", path))
        .to_string_lossy()
        .split('_')
        .next()
        .map(|s| Ok(s.replace('-', "")))
        .unwrap_or_else(|| Err(Error::UnknownMigrationFormat(path.to_path_buf())))
}
