use rstest::{fixture, rstest};
use std::net::TcpListener;

#[fixture]
fn app_address() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");
    // We retrieve the port assigned to us by the OS
    let port = listener.local_addr().unwrap().port();
    let server = zero2prod::run(listener).expect("Failed to bind address");
    let _ = tokio::spawn(server);
    // We return the application address to the caller!
    format!("http://127.0.0.1:{}", port)
}

#[fixture]
fn client() -> reqwest::Client {
    reqwest::Client::new()
}

#[rstest]
#[tokio::test]
async fn health_check_works(app_address: String, client: reqwest::Client) {
    // Act
    let response = client
        .get(&format!("{}/health_check", &app_address))
        .send()
        .await
        .expect("Failed to execute request.");
    // Assert
    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

#[rstest]
#[tokio::test]
async fn subscribe_returns_a_200_for_valid_form_data(app_address: String, client: reqwest::Client) {
    // Act
    let params = [("name", "le guin"), ("email", "ursula_le_guin@gmail.com")];
    let response = client
        .post(&format!("{}/subscriptions", &app_address))
        .form(&params)
        .send()
        .await
        .expect("Failed to execute request.");
    // Assert
    assert_eq!(200, response.status().as_u16());
}

#[rstest]
#[case(vec![("name","le guin")], "missing the email")]
#[case(vec![("email","ursula_le_guin@gmail.com")], "missing the email")]
#[case(vec![], "missing both name and email")]
#[tokio::test]
async fn subscribe_returns_a_400_when_data_is_missing(
    app_address: String,
    client: reqwest::Client,
    #[case] invalid_params: Vec<(&str, &str)>,
    #[case] error_message: String,
) {
    // Act
    let response = client
        .post(&format!("{}/subscriptions", &app_address))
        .form(&invalid_params)
        .send()
        .await
        .expect("Failed to execute request.");
    // Assert
    assert_eq!(
        400,
        response.status().as_u16(),
        // Additional customised error message on test failure
        "The API did not fail with 400 Bad Request when the payload was {}.",
        error_message
    );
}
