use std::net::TcpListener;

use newsletter::{
    configuration::get_configuration,
    startup::run,
};
use sqlx::{PgPool, Row};

pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool
}

#[tokio::test]
async fn health_check_works() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    let response = client
        .get(format!("{}/health_check", app.address))
        .send()
        .await
        .expect("Failed to execute request");

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length())
}

async fn spawn_app() -> TestApp {
    let listener = TcpListener::bind("localhost:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    println!("Test Server listening on port {port}");

    let config = get_configuration().unwrap();

    let connection_pool = PgPool::connect(&config.database.connection_string()).await.unwrap();

    let server = run(listener, connection_pool.clone()).unwrap();

    let _ = tokio::spawn(server);

    let address = format!("http://localhost:{port}");

    TestApp {
        address,
        db_pool: connection_pool
    }
}

#[tokio::test]
async fn subscribe_returns_a_200_for_valid_form_data() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";
    

    let response = client
        .post(format!("{}/subscriptions", &app.address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status().as_u16(), 200);

    let saved = sqlx::query("SELECT email, name FROM subscriptions")
        .fetch_one(&app.db_pool)
        .await
        .expect("Failed to fetch saved subscription.");

    assert_eq!(saved.get::<&str, _>("email"), "ursula_le_guin@gmail.com");
    assert_eq!(saved.get::<&str, _>("name"), "le guin");


    
}

#[tokio::test]
async fn subscribe_returns_a_400_for_invalid_form_data() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    let test_cases = vec![
        ("name=le%20guin", "missing the email"),
        ("email=ursula_le_guin%40gmail.com", "missing the name"),
        ("", "missing both name and email"),
    ];

    for (invalid_body, message) in test_cases {
        let response = client
            .post(format!("{}/subscriptions", &app.address))
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
