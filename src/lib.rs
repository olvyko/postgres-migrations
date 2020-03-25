pub use embed_migrations::*;
pub use file_migrations::*;

pub use shared::*;

#[macro_export]
/// This macro will read your migrations at compile time, and embed a module you can use to execute
/// them at runtime without the migration files being present on the file system.
///
/// You can optionally pass the path to the migrations directory to this macro. When left
/// unspecified, odegen will search for the migrations directory.
/// If specified, the path should be relative to the directory where `Cargo.toml` resides.
macro_rules! embed_migrations {
    () => {
        #[allow(dead_code)]
        mod embedded_migrations {
            use postgres_migrations::EmbedMigrations;

            #[derive(EmbedMigrations)]
            struct _Dummy;
        }
    };

    ($migrations_path:expr) => {
        #[allow(dead_code)]
        mod embedded_migrations {
            use postgres_migrations::EmbedMigrations;

            #[derive(EmbedMigrations)]
            #[embed_migrations_options(migrations_path=$migrations_path)]
            struct _Dummy;
        }
    };
}
