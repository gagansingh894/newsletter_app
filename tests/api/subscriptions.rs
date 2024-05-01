use wiremock::matchers::{method, path};
use wiremock::{Mock, ResponseTemplate};

use crate::helpers;

#[tokio::test]
async fn subscriber_returns_a_200_for_valid_form_data() {
    // Arrange
    let app = helpers::spawn_app().await;

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&app.email_server)
        .await;

    // Act
    let body = "name=jack%20reacher&email=jack_reacher%40gmail.com";
    let response = app.post_subscriptions(body.into()).await;

    // Assert
    assert_eq!(200, response.status().as_u16());
}

#[tokio::test]
async fn subscriber_persists_the_new_subscriber() {
    // Arrange
    let app = helpers::spawn_app().await;

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&app.email_server)
        .await;

    // Act
    let body = "name=jack%20reacher&email=jack_reacher%40gmail.com";
    let response = app.post_subscriptions(body.into()).await;

    // Assert
    assert_eq!(200, response.status().as_u16());

    let saved = sqlx::query!("SELECT email, name, status FROM subscriptions")
        .fetch_one(&app.db_pool)
        .await
        .expect("Failed to fetch saved subscription.");

    assert_eq!(saved.email, "jack_reacher@gmail.com");
    assert_eq!(saved.name, "jack reacher");
    assert_eq!(saved.status, "pending_confirmation");
}

#[tokio::test]
async fn subscriber_returns_a_400_when_fields_are_present_but_empty() {
    // Arrange
    let app = helpers::spawn_app().await;
    let test_cases = vec![
        ("name=&email=jack_reacher%40gmail.com", "empty name"),
        ("name=jack%20reacher&email=", "empty email"),
        (
            "name=jack%20reacher&email=definitely-not-an-email",
            "invalid email",
        ),
    ];

    for (body, description) in test_cases {
        // Act
        let response = app.post_subscriptions(body.into()).await;

        // Assert
        assert_eq!(
            400,
            response.status().as_u16(),
            "The API did not return a 200 OK response when the payload was {}",
            description
        )
    }
}

#[tokio::test]
async fn subscriber_returns_a_400_when_data_is_missing() {
    // Arrange
    let app = helpers::spawn_app().await;
    let test_cases = vec![
        ("name=jack%20reacher", "missing the email"),
        ("email=jack_reacher%40gmail.com", "missing the name"),
        ("", "missing both name and email"),
    ];

    for (invalid_body, error_message) in test_cases {
        // Act
        let response = app.post_subscriptions(invalid_body.into()).await;

        // Assert
        assert_eq!(
            400,
            response.status().as_u16(),
            "The API did not fail with 400 Bad Request when the payload was {}.",
            error_message
        );
    }
}

#[tokio::test]
async fn subscriber_sends_a_confirmation_email_for_valid_data() {
    // Arrange
    let app = helpers::spawn_app().await;
    let body = "name=jack%20reacher&email=jack_reacher%40gmail.com";

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&app.email_server)
        .await;

    // Act
    let response = app.post_subscriptions(body.into()).await;

    // Assert
    assert_eq!(200, response.status().as_u16());
}

#[tokio::test]
async fn subscribe_sends_confirmation_email_with_a_link() {
    // Arrange
    let app = helpers::spawn_app().await;
    let body = "name=jack%20reacher&email=jack_reacher%40gmail.com";

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&app.email_server)
        .await;

    // Act
    app.post_subscriptions(body.into()).await;

    // Assert
    let email_request = &app.email_server.received_requests().await.unwrap()[0];
    let confirmation_links = &app.get_confirmation_links(email_request);

    assert_eq!(confirmation_links.html, confirmation_links.plain_text);
}
