use log::LevelFilter;
use sqlx::mysql::{MySqlConnectOptions, MySqlPoolOptions};
use sqlx::{ConnectOptions, Executor, MySql, Pool};
use std::str::FromStr;

pub async fn get_db(dsn: &str, is_log: bool) -> Result<Pool<MySql>, sqlx::Error> {
    //  for MySQL, use MySqlPoolOptions::new()
    let mut connection_options = MySqlConnectOptions::from_str(dsn)?;
    if is_log {
        connection_options.log_statements(LevelFilter::Info);
    } else {
        connection_options.disable_statement_logging();
    }
    MySqlPoolOptions::new()
        .max_connections(1)
        .after_connect(|conn, _meta| {
            Box::pin(async move {
                let _ = conn.execute("SET time_zone='SYSTEM'").await;
                Ok(())
            })
        })
        .connect_with(connection_options)
        .await
}

pub async fn get_db_by_default(
    host: &str,
    port: i16,
    username: &str,
    password: &str,
    database: &str,
    is_log: bool,
) -> Result<Pool<MySql>, sqlx::Error> {
    let dsn = format!(
        "mysql://{username}:{password}@{host}:{port}/{database}",
        username = username,
        password = password,
        host = host,
        port = port,
        database = database,
    );

    get_db(&dsn, is_log).await
}
