use shared::async_trait;
use shared::error::*;
use shared::migration::*;
use shared::run_migrations;
use shared::runner::*;
use std::fs::File;
use std::io::{stdout, Read, Write};
use std::path::{Path, PathBuf};

/// Runs all migrations that have not yet been run. This function will print all progress to
/// stdout. This function will return an `Err` if some error occurs reading the migrations, or if
/// any migration fails to run. Each migration is run in its own transaction, so some migrations
/// may be committed, even if a later migration fails to run.
///
/// It should be noted that this runs all migrations that have not already been run, regardless of
/// whether or not their version is later than the latest run migration. This is generally not a
/// problem, and eases the more common case of two developers generating independent migrations on
/// a branch. Whoever created the second one will eventually need to run the first when both
/// branches are merged.
pub async fn run_pending_migrations(pool: DbConnectionPool) -> Result<(), Error> {
    let migrations_dir = shared::find_migrations_directory()?;
    run_pending_migrations_in_directory(pool, &migrations_dir, &mut stdout()).await
}

#[doc(hidden)]
pub async fn run_pending_migrations_in_directory(
    pool: DbConnectionPool,
    migrations_dir: &Path,
    output: &mut (dyn Write + Send + Sync),
) -> Result<(), Error> {
    let all_migrations = migrations_in_directory(migrations_dir)?;
    run_migrations(pool, all_migrations, output).await
}

fn migrations_in_directory(path: &Path) -> Result<Vec<Box<dyn Migration + Send + Sync>>, Error> {
    shared::migration_paths_in_directory(path)?
        .iter()
        .map(|e| migration_from(e.path()))
        .collect()
}

pub fn migration_from(path: PathBuf) -> Result<Box<dyn Migration + Send + Sync>, Error> {
    if valid_sql_migration_directory(&path) {
        let version = shared::version_from_path(&path)?;
        Ok(Box::new(SqlFileMigration(path, version)))
    } else {
        Err(Error::UnknownMigrationFormat(path))
    }
}

fn valid_sql_migration_directory(path: &Path) -> bool {
    file_names(path)
        .map(|files| files.contains(&"down.sql".into()) && files.contains(&"up.sql".into()))
        .unwrap_or(false)
}

fn file_names(path: &Path) -> Result<Vec<String>, Error> {
    path.read_dir()?
        .map(|entry| {
            let file_name = entry?.file_name();

            // FIXME(killercup): Decide whether to add Error variant for this
            match file_name.into_string() {
                Ok(utf8_file_name) => Ok(utf8_file_name),
                Err(original_os_string) => {
                    panic!("Can't convert file name `{:?}` into UTF8 string", original_os_string)
                }
            }
        })
        .filter(|file_name| match *file_name {
            Ok(ref name) => !name.starts_with('.'),
            _ => true,
        })
        .collect()
}

pub struct SqlFileMigration(pub PathBuf, pub String);

#[async_trait]
impl Migration for SqlFileMigration {
    fn version(&self) -> &str {
        &self.1
    }

    async fn run(&self, transaction: &DbTransaction<'_>) -> Result<(), Error> {
        run_sql_from_file(transaction, &self.0.join("up.sql")).await
    }

    async fn revert(&self, transaction: &DbTransaction<'_>) -> Result<(), Error> {
        run_sql_from_file(transaction, &self.0.join("down.sql")).await
    }

    fn file_path(&self) -> Option<&Path> {
        Some(self.0.as_path())
    }
}

async fn run_sql_from_file(transaction: &DbTransaction<'_>, path: &Path) -> Result<(), Error> {
    let mut sql = String::new();
    let mut file = File::open(path)?;
    file.read_to_string(&mut sql)?;
    if sql.is_empty() {
        return Err(Error::EmptyMigration);
    }
    transaction.batch_execute(&sql).await?;
    Ok(())
}
