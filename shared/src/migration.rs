use crate::runner::*;
use crate::error::*;
use async_trait::async_trait;
use std::fmt;
use std::path::Path;

#[async_trait]
pub trait Migration {
    /// Get the migration version
    fn version(&self) -> &str;
    /// Apply this migration
    async fn run(&self, transaction: &DbTransaction<'_>) -> Result<(), Error>;
    /// Revert this migration
    async fn revert(&self, transaction: &DbTransaction<'_>) -> Result<(), Error>;
    /// Get the migration file path
    fn file_path(&self) -> Option<&Path> {
        None
    }
}

#[async_trait]
impl Migration for Box<dyn Migration + Send + Sync> {
    fn version(&self) -> &str {
        (&**self).version()
    }

    async fn run(&self, transaction: &DbTransaction<'_>) -> Result<(), Error> {
        (&**self).run(transaction).await
    }

    async fn revert(&self, transaction: &DbTransaction<'_>) -> Result<(), Error> {
        (&**self).revert(transaction).await
    }

    fn file_path(&self) -> Option<&Path> {
        (&**self).file_path()
    }
}

#[async_trait]
impl<'a> Migration for &'a (dyn Migration + Send + Sync) {
    fn version(&self) -> &str {
        (&**self).version()
    }

    async fn run(&self, transaction: &DbTransaction<'_>) -> Result<(), Error> {
        (&**self).run(transaction).await
    }

    async fn revert(&self, transaction: &DbTransaction<'_>) -> Result<(), Error> {
        (&**self).revert(transaction).await
    }

    fn file_path(&self) -> Option<&Path> {
        (&**self).file_path()
    }
}

#[derive(Clone, Copy)]
pub struct MigrationName<'a> {
    pub migration: &'a dyn Migration,
}

pub fn name(migration: &dyn Migration) -> MigrationName {
    MigrationName { migration }
}

impl<'a> fmt::Display for MigrationName<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let file_name = self
            .migration
            .file_path()
            .and_then(|file_path| file_path.file_name()?.to_str());
        if let Some(name) = file_name {
            f.write_str(name)?;
        } else {
            f.write_str(self.migration.version())?;
        }
        Ok(())
    }
}

#[derive(Clone, Copy)]
pub struct MigrationFileName<'a> {
    pub migration: &'a dyn Migration,
    pub sql_file: &'a str,
}

pub fn file_name<'a>(migration: &'a dyn Migration, sql_file: &'a str) -> MigrationFileName<'a> {
    MigrationFileName { migration, sql_file }
}

impl<'a> fmt::Display for MigrationFileName<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(path) = self.migration.file_path() {
            let fpath = path.join(self.sql_file);
            f.write_str(fpath.to_str().unwrap_or("Invalid utf8 in filename"))
        } else {
            write!(f, "{}/{}", self.migration.version(), self.sql_file)
        }
    }
}
