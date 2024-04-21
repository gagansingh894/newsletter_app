use std::net::TcpListener;

use secrecy::ExposeSecret;
use sqlx::postgres::PgPoolOptions;

use newsletter_app::configuration::get_configuration;
use newsletter_app::startup::run;
use newsletter_app::telemetry::{get_subscriber, init_subscriber};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let subscriber = get_subscriber("newsletter_app".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    let configuration = get_configuration().expect("failed to read configuration");
    let connection_pool = PgPoolOptions::new()
        .idle_timeout(std::time::Duration::from_secs(2))
        .connect_lazy(&configuration.database.connection_string().expose_secret())
        .expect("Failed to connect to Postgres.");
    let address = format!("{}:{}", configuration.application.host, configuration.application.port);
    let listener = TcpListener::bind(address)?;
    run(listener, connection_pool)?.await?;
    Ok(())
}
