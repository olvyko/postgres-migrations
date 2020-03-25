use std::io;
use std::path::PathBuf;
use thiserror::*;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Unable to find migrations directory in {0:?} or any parent directories.")]
    MigrationDirectoryNotFound(PathBuf),
    #[error(
        "Invalid migration directory, the directory's name should be \
        <timestamp>_<name_of_migration>, and it should only contain up.sql and down.sql."
    )]
    UnknownMigrationFormat(PathBuf),
    #[error("Unable to find migration version to revert in the migrations directory.")]
    UnknownMigrationVersion(String),
    #[error("No migrations have been run. Did you forget `migration run`?")]
    NoMigrationRun,
    #[error("Failed with: Attempted to run an empty migration.")]
    EmptyMigration,

    #[error(transparent)]
    IoError(#[from] io::Error),
    #[error(transparent)]
    Postgres(#[from] tokio_postgres::Error),
    #[error(transparent)]
    Bb8(#[from] bb8_postgres::bb8::RunError<tokio_postgres::Error>),
}
