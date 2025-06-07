use std::net::TcpListener;

use newsletter::{
    configuration::{self, get_configuration},
    startup::run,
};
use sqlx::{Connection, PgConnection, Row};

#[tokio::test]

async fn health_check_works() {
    let url = spawn_app();

    let client = reqwest::Client::new();

    let response = client
        .get(format!("{url}/health_check"))
        .send()
        .await
        .expect("Failed to execute request");

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length())
}

fn spawn_app() -> String {
    let listener = TcpListener::bind("localhost:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    println!("Test Server listening on port {port}");

    let configuration = configuration::get_configuration().unwrap();
    let connection = sqlx::PgConnection::connect(&configuration.database.connection_string())
        .await
        .unwrap();

    let server = run(listener).unwrap();

    let _ = tokio::spawn(server);

    format!("http://localhost:{port}")
}

#[tokio::test]
async fn subscribe_returns_a_200_for_valid_form_data() {
    let app_address = spawn_app();
    let client = reqwest::Client::new();

    let config = get_configuration().unwrap();
    let connection_string = config.database.connection_string();

    let mut connection = PgConnection::connect(&connection_string).await.unwrap();
    let saved = sqlx::query("SELECT email, name FROM subscriptions")
        .fetch_one(&mut connection)
        .await
        .expect("Failed to fetch saved subscription.");

    assert_eq!(saved.get::<&str, _>("email"), "ursula_le_guin@gmail.com");
    assert_eq!(saved.get::<&str, _>("name"), "le guin");

    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";

    let response = client
        .post(format!("{}/subscriptions", &app_address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status().as_u16(), 200);
}

#[tokio::test]
async fn subscribe_returns_a_400_for_invalid_form_data() {
    let app_address = spawn_app();
    let client = reqwest::Client::new();

    let test_cases = vec![
        ("name=le%20guin", "missing the email"),
        ("email=ursula_le_guin%40gmail.com", "missing the name"),
        ("", "missing both name and email"),
    ];

    for (invalid_body, message) in test_cases {
        let response = client
            .post(format!("{}/subscriptions", &app_address))
            .body(invalid_body)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .send()
            .await
            .expect("Failed to execute request");

        assert_eq!(
            400,
            response.status().as_u16(),
            "The API did not fail with 400 bad request when the payload is {}",
            message
        );
    }
}
