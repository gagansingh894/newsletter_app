use newsletter_app::configuration::get_configuration;
use newsletter_app::startup::Application;
use newsletter_app::telemetry::{get_subscriber, init_subscriber};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    // initialize tracing
    let subscriber = get_subscriber("newsletter_app".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    // read configuration
    let configuration = get_configuration().expect("failed to read configuration");

    // build application
    let application = Application::build(configuration).await?;
    application.run_until_stopped().await?;
    Ok(())
}
