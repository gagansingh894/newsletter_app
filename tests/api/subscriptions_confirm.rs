use std::fmt::Debug;

use wiremock::matchers::{method, path};
use wiremock::{Mock, ResponseTemplate};

use crate::helpers;
use crate::helpers::spawn_app;

#[tokio::test]
async fn confirmation_without_a_token_are_rejected_with_400() {
    // Arrange
    let app = helpers::spawn_app().await;

    // Act
    let response = reqwest::get(&format!("{}/subscriptions/confirm", app.address))
        .await
        .unwrap();

    // Assert
    assert_eq!(response.status().as_u16(), 400);
}

#[tokio::test]
async fn the_link_returned_by_subscribe_returns_200_if_called() {
    // Arrange
    let app = spawn_app().await;
    let body = "name=jack%20reacher&email=jack_reacher%40gmail.com";

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&app.email_server)
        .await;

    app.post_subscriptions(body.into()).await;

    // extract the link
    let email_request = &app.email_server.received_requests().await.unwrap()[0];
    let confirmation_links = app.get_confirmation_links(&email_request);

    // Act
    let response = reqwest::get(confirmation_links.html).await.unwrap();

    // Assert
    assert_eq!(response.status().as_u16(), 200)
}