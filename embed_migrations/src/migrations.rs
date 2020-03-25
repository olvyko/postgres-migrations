use std::env;
use std::error::Error;
use std::path::{Path, PathBuf};

pub fn migration_directory_from_given_path(given_path: Option<&str>) -> Result<PathBuf, Box<dyn Error>> {
    let cargo_toml_directory = env::var("CARGO_MANIFEST_DIR")?;
    let cargo_manifest_path = Path::new(&cargo_toml_directory);
    let migrations_path = given_path.as_ref().map(Path::new);
    resolve_migrations_directory(cargo_manifest_path, migrations_path)
}

fn resolve_migrations_directory(
    cargo_manifest_dir: &Path,
    relative_path_to_migrations: Option<&Path>,
) -> Result<PathBuf, Box<dyn Error>> {
    let result = match relative_path_to_migrations {
        Some(dir) => cargo_manifest_dir.join(dir),
        None => {
            // People commonly put their migrations in src/migrations
            // so start the search there rather than the project root
            let src_dir = cargo_manifest_dir.join("src");
            shared::search_for_migrations_directory(&src_dir)?
        }
    };

    result.canonicalize().map_err(Into::into)
}
