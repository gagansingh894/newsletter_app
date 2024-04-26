use std::net::TcpListener;

use sqlx::postgres::PgPoolOptions;

use newsletter_app::configuration::get_configuration;
use newsletter_app::email_client::EmailClient;
use newsletter_app::startup::run;
use newsletter_app::telemetry::{get_subscriber, init_subscriber};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    // initialize tracing
    let subscriber = get_subscriber("newsletter_app".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    // read configuration
    let configuration = get_configuration().expect("failed to read configuration");

    // create connection pool for database
    let connection_pool = PgPoolOptions::new()
        .idle_timeout(std::time::Duration::from_secs(2))
        .connect_lazy_with(configuration.database.with_db());

    // build EmailClient using configuration
    let sender_email = configuration.email_client.sender().expect("Invalid sender email address");
    let email_client = EmailClient::new(configuration.email_client.base_url, sender_email);

    // build address for application server
    let address = format!(
        "{}:{}",
        configuration.application.host, configuration.application.port
    );
    let listener = TcpListener::bind(address)?;

    // run the server
    run(listener, connection_pool, email_client)?.await?;
    Ok(())
}
