use proc_macro2;
use syn;

use crate::migrations::migration_directory_from_given_path;
use crate::util;
use std::error::Error;
use std::fs::DirEntry;
use std::path::Path;

pub fn derive_embed_migrations(input: &syn::DeriveInput) -> proc_macro2::TokenStream {
    fn bug() -> ! {
        panic!(
            "This is a bug. Please open a Github issue \
             with your invocation of `embed_migrations!"
        );
    }

    let options = util::get_options_from_input(&parse_quote!(embed_migrations_options), &input.attrs, bug);
    let migrations_path_opt = options.as_ref().map(|o| util::get_option(o, "migrations_path", bug));
    let migrations_expr = migration_directory_from_given_path(migrations_path_opt.as_ref().map(String::as_str))
        .and_then(|path| migration_literals_from_path(&path));
    let migrations_expr = match migrations_expr {
        Ok(v) => v,
        Err(e) => panic!("Error reading migrations: {}", e),
    };

    // These are split into multiple `quote!` calls to avoid recursion limit
    let embedded_migration_def = quote!(
        struct EmbeddedMigration {
            version: &'static str,
            up_sql: &'static str,
        }

        #[async_trait]
        impl Migration for EmbeddedMigration {
            fn version(&self) -> &str {
                self.version
            }

            async fn run(&self, transaction: &DbTransaction<'_>) -> Result<(), RunMigrationsError> {
                transaction.batch_execute(self.up_sql).await?;
                Result::<(), RunMigrationsError>::Ok(())
            }

            async fn revert(&self, _transaction: &DbTransaction<'_>) -> Result<(), RunMigrationsError> {
                unreachable!()
            }
        }
    );

    let run_fns = quote!(
        pub async fn run(pool: DbConnectionPool) -> Result<(), RunMigrationsError> {
            run_with_output(pool, &mut io::sink()).await
        }

        pub async fn run_with_output(
            pool: DbConnectionPool,
            out: &mut (dyn io::Write + Send + Sync),
        ) -> Result<(), RunMigrationsError> {
            run_migrations(pool, ALL_MIGRATIONS.iter().map(|v| *v).collect(), out).await
        }
    );

    quote! {
        use postgres_migrations::*;
        use postgres_migrations::migration::{Migration};
        use postgres_migrations::error::Error as RunMigrationsError;
        use postgres_migrations::runner::{DbTransaction, DbConnectionPool};

        use std::io;

        const ALL_MIGRATIONS: &[&(Migration + Send + Sync)] = &[#(#migrations_expr),*];

        #embedded_migration_def

        #run_fns
    }
}

fn migration_literals_from_path(path: &Path) -> Result<Vec<proc_macro2::TokenStream>, Box<dyn Error>> {
    let mut migrations = shared::migration_paths_in_directory(path)?;

    migrations.sort_by_key(DirEntry::path);

    migrations
        .into_iter()
        .map(|e| migration_literal_from_path(&e.path()))
        .collect()
}

fn migration_literal_from_path(path: &Path) -> Result<proc_macro2::TokenStream, Box<dyn Error>> {
    let version = shared::version_from_path(path)?;
    let sql_file = path.join("up.sql");
    let sql_file_path = sql_file.to_str();

    Ok(quote!(&EmbeddedMigration {
        version: #version,
        up_sql: include_str!(#sql_file_path),
    }))
}
