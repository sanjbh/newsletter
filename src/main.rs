use std::net::TcpListener;

use anyhow::{Ok, Result};
use newsletter::{configuration, startup::run};
use sqlx::Connection;

#[tokio::main]
async fn main() -> Result<()> {
    let configuration = configuration::get_configuration()?;
    let listener = TcpListener::bind(format!("localhost:{}", configuration.application_port))?;
    let pg_pool = sqlx::PgPool::connect(&configuration.database.connection_string()).await?;
    let server = run(listener, pg_pool)?;

    Ok(server.await?)

    // Ok(())
}

#[cfg(test)]
mod tests {
    use newsletter::routes::health_check;

    #[tokio::test]
    async fn health_check_succeeds() {
        let response = health_check().await;
        assert!(response.status().is_success())
    }
}
