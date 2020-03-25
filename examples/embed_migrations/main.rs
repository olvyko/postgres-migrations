use bb8_postgres::{bb8::Pool, PostgresConnectionManager};
use postgres_migrations::embed_migrations;
use std::time::Duration;
use tokio_postgres::{Config as PostgresConfig, NoTls};

embed_migrations!("examples/migrations");

#[tokio::main]
async fn main() {
    let mut pg_config = PostgresConfig::new();
    pg_config
        .user("user")
        .password("pass")
        .dbname("server")
        .host("localhost")
        .port(5432);
    let manager = PostgresConnectionManager::new(pg_config, NoTls);
    let pool = Pool::builder()
        .connection_timeout(Duration::from_secs(2))
        .build(manager)
        .await
        .unwrap();

    embedded_migrations::run(pool.clone()).await.unwrap();
}
