mod common;

#[tokio::test]
async fn subscriber_returns_a_200_for_valid_form_data() {
    // Arrange
    let app = common::spawn_app().await;

    // Act
    let body = "name=jack%20reacher&email=jack_reacher%40gmail.com";
    let response = app.post_subscriptions(body.into()).await;

    // Assert
    assert_eq!(200, response.status().as_u16());

    let saved = sqlx::query!("SELECT email, name FROM subscriptions")
        .fetch_one(&app.db_pool)
        .await
        .expect("Failed to fetch saved subscription.");

    assert_eq!(saved.email, "jack_reacher@gmail.com");
    assert_eq!(saved.name, "jack reacher");
}

#[tokio::test]
async fn subscriber_returns_a_400_when_fields_are_present_but_empty() {
    // Arrange
    let app = common::spawn_app().await;
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
    let app = common::spawn_app().await;
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
