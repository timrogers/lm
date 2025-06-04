use lm_rs::{ApiClient, AuthenticationClient, Credentials, LaMarzoccoClient, TokenRefreshCallback};
use std::sync::Arc;
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn test_full_authentication_flow_with_mock_server() {
    // Start a mock server
    let mock_server = MockServer::start().await;

    // Mock the authentication endpoint
    Mock::given(method("POST"))
        .and(path("/auth/signin"))
        .and(header("content-type", "application/json"))
        .respond_with(
            ResponseTemplate::new(200).set_body_string(include_str!("fixtures/auth_success.json")),
        )
        .mount(&mock_server)
        .await;

    // Create client with mock server URL
    let mut client = LaMarzoccoClient::new_with_base_url(mock_server.uri());

    // Test authentication
    let result = client.authenticate("test@example.com", "password123").await;
    assert!(result.is_ok());
    assert!(client.access_token().is_some());
}

#[tokio::test]
async fn test_authentication_failure_with_mock_server() {
    let mock_server = MockServer::start().await; // Mock failed authentication
    Mock::given(method("POST"))
        .and(path("/auth/signin"))
        .respond_with(
            ResponseTemplate::new(401).set_body_string(include_str!("fixtures/auth_failure.json")),
        )
        .mount(&mock_server)
        .await;

    let mut client = LaMarzoccoClient::new_with_base_url(mock_server.uri());

    let result = client
        .authenticate("test@example.com", "wrongpassword")
        .await;
    assert!(result.is_err());
    assert!(client.access_token().is_none());
}

#[tokio::test]
async fn test_get_machines_with_mock_server() {
    let mock_server = MockServer::start().await; // Mock authentication
    Mock::given(method("POST"))
        .and(path("/auth/signin"))
        .respond_with(
            ResponseTemplate::new(200).set_body_string(include_str!("fixtures/auth_success.json")),
        )
        .mount(&mock_server)
        .await;

    // Mock machines endpoint
    Mock::given(method("GET"))
        .and(path("/things"))
        .respond_with(
            ResponseTemplate::new(200).set_body_string(include_str!("fixtures/machines.json")),
        )
        .mount(&mock_server)
        .await;

    let mut client = LaMarzoccoClient::new_with_base_url(mock_server.uri());

    // Authenticate first
    client
        .authenticate("test@example.com", "password123")
        .await
        .unwrap();

    // Get machines
    let machines = client.get_machines().await.unwrap();
    assert_eq!(machines.len(), 2);
    assert_eq!(machines[0].serial_number, "MR033274");
    assert_eq!(machines[1].serial_number, "GS001234");
    assert_eq!(machines[0].name, Some("Linea Micra".to_string()));
    assert_eq!(machines[1].name, Some("Office Machine".to_string()));
    assert!(machines[0].connected);
    assert!(!machines[1].connected);
}

#[tokio::test]
async fn test_get_machine_status_with_mock_server() {
    let mock_server = MockServer::start().await; // Mock authentication
    Mock::given(method("POST"))
        .and(path("/auth/signin"))
        .respond_with(
            ResponseTemplate::new(200).set_body_string(include_str!("fixtures/auth_success.json")),
        )
        .mount(&mock_server)
        .await;

    // Mock machine status endpoint
    Mock::given(method("GET"))
        .and(path("/things/MR033274/dashboard"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string(include_str!("fixtures/machine_status_on.json")),
        )
        .mount(&mock_server)
        .await;

    let mut client = LaMarzoccoClient::new_with_base_url(mock_server.uri());

    // Authenticate first
    client
        .authenticate("test@example.com", "password123")
        .await
        .unwrap();

    // Get machine status
    let status = client.get_machine_status("MR033274").await.unwrap();
    assert!(status.is_on());
}

#[tokio::test]
async fn test_turn_on_machine_with_mock_server() {
    let mock_server = MockServer::start().await;

    // Mock authentication
    Mock::given(method("POST"))
        .and(path("/auth/signin"))
        .respond_with(
            ResponseTemplate::new(200).set_body_string(include_str!("fixtures/auth_success.json")),
        )
        .mount(&mock_server)
        .await;

    // Mock machine command endpoint
    Mock::given(method("POST"))
        .and(path("/things/MR033274/command/CoffeeMachineChangeMode"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string(include_str!("fixtures/machine_command_success.json")),
        )
        .mount(&mock_server)
        .await;

    let mut client = LaMarzoccoClient::new_with_base_url(mock_server.uri());

    // Authenticate first
    client
        .authenticate("test@example.com", "password123")
        .await
        .unwrap();

    // Turn on machine
    let result = client.turn_on_machine("MR033274").await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_turn_off_machine_with_mock_server() {
    let mock_server = MockServer::start().await;

    // Mock authentication
    Mock::given(method("POST"))
        .and(path("/auth/signin"))
        .respond_with(
            ResponseTemplate::new(200).set_body_string(include_str!("fixtures/auth_success.json")),
        )
        .mount(&mock_server)
        .await;

    // Mock machine command endpoint
    Mock::given(method("POST"))
        .and(path("/things/MR033274/command/CoffeeMachineChangeMode"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string(include_str!("fixtures/machine_command_success.json")),
        )
        .mount(&mock_server)
        .await;

    let mut client = LaMarzoccoClient::new_with_base_url(mock_server.uri());

    // Authenticate first
    client
        .authenticate("test@example.com", "password123")
        .await
        .unwrap();

    // Turn off machine
    let result = client.turn_off_machine("MR033274").await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_machine_command_error_with_mock_server() {
    let mock_server = MockServer::start().await;

    // Mock authentication
    Mock::given(method("POST"))
        .and(path("/auth/signin"))
        .respond_with(
            ResponseTemplate::new(200).set_body_string(include_str!("fixtures/auth_success.json")),
        )
        .mount(&mock_server)
        .await;

    // Mock machine command error
    Mock::given(method("POST"))
        .and(path("/things/INVALID123/command/CoffeeMachineChangeMode"))
        .respond_with(
            ResponseTemplate::new(404)
                .set_body_string(include_str!("fixtures/machine_command_error.json")),
        )
        .mount(&mock_server)
        .await;

    let mut client = LaMarzoccoClient::new_with_base_url(mock_server.uri());

    // Authenticate first
    client
        .authenticate("test@example.com", "password123")
        .await
        .unwrap();

    // Try to turn on invalid machine
    let result = client.turn_on_machine("INVALID123").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_json_fixture_parsing() {
    use lm_rs::types::{Machine, MachineStatus};
    use serde_json;

    // Test machines JSON parsing
    let machines: Vec<Machine> =
        serde_json::from_str(include_str!("fixtures/machines.json")).unwrap();
    assert_eq!(machines.len(), 2);
    assert_eq!(machines[0].serial_number, "MR033274");
    assert_eq!(machines[0].model, Some("LINEA MICRA".to_string()));
    assert_eq!(machines[0].name, Some("Linea Micra".to_string()));
    assert!(machines[0].connected);

    // Test machine status JSON parsing
    let status_on: MachineStatus =
        serde_json::from_str(include_str!("fixtures/machine_status_on.json")).unwrap();
    assert!(status_on.is_on());
    assert_eq!(status_on.get_status_string(), "On (Ready)");

    let status_standby: MachineStatus =
        serde_json::from_str(include_str!("fixtures/machine_status_standby.json")).unwrap();
    assert!(!status_standby.is_on());
    assert_eq!(status_standby.get_status_string(), "Standby");

    let status_warming: MachineStatus =
        serde_json::from_str(include_str!("fixtures/machine_status_warming.json")).unwrap();
    assert!(status_warming.is_on());

    // Test with a fixed current time to avoid flaky tests
    let fixed_current_time = 1748515647000; // 5 minutes before ready time
    let warming_status = status_warming.get_status_string_with_time(Some(fixed_current_time));
    assert_eq!(warming_status, "On (Ready in 5 mins)");
}

// --- NEW LIBRARY INTERFACE TESTS ---

/// Test token refresh callback implementation
struct TestTokenCallback {
    pub refreshed: std::sync::Arc<std::sync::Mutex<bool>>,
}

impl TestTokenCallback {
    fn new() -> Self {
        Self {
            refreshed: std::sync::Arc::new(std::sync::Mutex::new(false)),
        }
    }
}

impl TokenRefreshCallback for TestTokenCallback {
    fn on_tokens_refreshed(&self, credentials: &Credentials) {
        let mut refreshed = self.refreshed.lock().unwrap();
        *refreshed = true;
        println!(
            "Test callback: tokens refreshed for {}",
            credentials.username
        );
    }
}

#[tokio::test]
async fn test_new_authentication_client_with_mock_server() {
    // Start a mock server
    let mock_server = MockServer::start().await;

    // Mock the authentication endpoint with refresh token
    Mock::given(method("POST"))
        .and(path("/auth/signin"))
        .and(header("content-type", "application/json"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string(include_str!("fixtures/auth_success_with_refresh.json")),
        )
        .mount(&mock_server)
        .await;

    // Create authentication client with mock server URL
    let auth_client = AuthenticationClient::new_with_base_url(mock_server.uri());

    // Test authentication
    let result = auth_client.login("test@example.com", "password123").await;
    assert!(result.is_ok());

    let tokens = result.unwrap();
    assert_eq!(tokens.username, "test@example.com");
    assert!(!tokens.access_token.is_empty());
    assert!(!tokens.refresh_token.is_empty());
}

#[tokio::test]
async fn test_new_api_client_with_machines_with_mock_server() {
    // Start a mock server
    let mock_server = MockServer::start().await;

    // Mock the machines endpoint - use a simple token since JWT validation requires proper signature
    Mock::given(method("GET"))
        .and(path("/things"))
        .and(header("authorization", "Bearer simple_test_token"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string(include_str!("fixtures/machines_response.json")),
        )
        .mount(&mock_server)
        .await;

    // Create tokens - use a simple token for testing
    let tokens = Credentials {
        access_token: "simple_test_token".to_string(),
        refresh_token: "test_refresh_token".to_string(),
        username: "test@example.com".to_string(),
    };

    // Create callback
    let callback = Arc::new(TestTokenCallback::new());

    // Create API client
    let mut api_client =
        ApiClient::new_with_base_url(tokens, Some(callback.clone()), mock_server.uri());

    // Test getting machines
    let result = api_client.get_machines().await;
    assert!(result.is_ok());

    let machines = result.unwrap();
    assert!(!machines.is_empty());
    assert_eq!(machines[0].serial_number, "GS01234");
}

#[tokio::test]
async fn test_new_api_client_machine_operations_with_mock_server() {
    // Start a mock server
    let mock_server = MockServer::start().await;

    // Mock machine status endpoint - use simple token
    Mock::given(method("GET"))
        .and(path("/things/GS01234/dashboard"))
        .and(header("authorization", "Bearer simple_test_token"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string(include_str!("fixtures/machine_status_ready.json")),
        )
        .mount(&mock_server)
        .await;

    // Mock machine command endpoint - use simple token
    Mock::given(method("POST"))
        .and(path("/things/GS01234/command/CoffeeMachineChangeMode"))
        .and(header("authorization", "Bearer simple_test_token"))
        .respond_with(ResponseTemplate::new(200).set_body_string("{}"))
        .mount(&mock_server)
        .await;

    // Create tokens - use simple token for testing
    let tokens = Credentials {
        access_token: "simple_test_token".to_string(),
        refresh_token: "test_refresh_token".to_string(),
        username: "test@example.com".to_string(),
    };

    // Create API client
    let mut api_client = ApiClient::new_with_base_url(tokens, None, mock_server.uri());

    // Test getting machine status
    let status_result = api_client.get_machine_status("GS01234").await;
    assert!(status_result.is_ok());

    let status = status_result.unwrap();
    assert!(status.is_on());

    // Test turning on machine
    let turn_on_result = api_client.turn_on_machine("GS01234").await;
    assert!(turn_on_result.is_ok());

    // Test turning off machine
    let turn_off_result = api_client.turn_off_machine("GS01234").await;
    assert!(turn_off_result.is_ok());
}

#[tokio::test]
async fn test_token_refresh_callback() {
    // Create tokens
    let credentials = Credentials {
        access_token: "test_access_token".to_string(),
        refresh_token: "test_refresh_token".to_string(),
        username: "test@example.com".to_string(),
    };

    // Create callback
    let callback = Arc::new(TestTokenCallback::new());

    // Manually trigger callback (simulating a token refresh)
    callback.on_tokens_refreshed(&credentials);

    // Check that callback was called
    let refreshed = callback.refreshed.lock().unwrap();
    assert!(*refreshed);
}

#[tokio::test]
async fn test_jwt_token_expiration_function() {
    // Test with malformed JWT token (starts with "ey" but invalid)
    let invalid_jwt_token = "eyJhbGciOiJIUzUxMiJ9.invalid.token";
    assert!(lm_rs::is_token_expired(invalid_jwt_token, 0));

    // Test with non-JWT test token (should not be expired for testing)
    let test_token = "simple_test_token";
    assert!(!lm_rs::is_token_expired(test_token, 0));

    // Test with empty token
    let empty_token = "";
    assert!(!lm_rs::is_token_expired(empty_token, 0)); // Empty is considered a test token
}

#[tokio::test]
async fn test_authentication_client_token_refresh_with_mock_server() {
    // Start a mock server
    let mock_server = MockServer::start().await;

    // Mock the token refresh endpoint
    Mock::given(method("POST"))
        .and(path("/auth/refreshtoken"))
        .and(header("content-type", "application/json"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string(include_str!("fixtures/auth_refresh_success.json")),
        )
        .mount(&mock_server)
        .await;

    // Create authentication client with mock server URL
    let auth_client = AuthenticationClient::new_with_base_url(mock_server.uri());

    // Test token refresh
    let result = auth_client.refresh_token("refresh_token_123").await;
    assert!(result.is_ok());

    let tokens = result.unwrap();
    assert!(!tokens.access_token.is_empty());
    assert_eq!(tokens.refresh_token, "new_refresh_token_789".to_string());
    assert_eq!(tokens.username, "test@example.com"); // Extracted from JWT
}

#[tokio::test]
async fn test_api_client_automatic_token_refresh_with_mock_server() {
    // Start a mock server
    let mock_server = MockServer::start().await;

    // Mock the token refresh endpoint
    Mock::given(method("POST"))
        .and(path("/auth/refreshtoken"))
        .and(header("content-type", "application/json"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string(include_str!("fixtures/auth_refresh_success.json")),
        )
        .mount(&mock_server)
        .await;

    // Mock the machines endpoint for the NEW token
    Mock::given(method("GET"))
        .and(path("/things"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string(include_str!("fixtures/machines_response.json")),
        )
        .mount(&mock_server)
        .await;

    // Create tokens with an expired access token (JWT that starts with "ey" but invalid will be considered expired)
    let tokens = Credentials {
        access_token: "eyJhbGciOiJIUzUxMiJ9.invalid.expired".to_string(), // This will be considered expired
        refresh_token: "refresh_token_123".to_string(),
        username: "test@example.com".to_string(),
    };

    // Create callback to verify refresh was called
    let callback = Arc::new(TestTokenCallback::new());

    // Create API client
    let mut api_client =
        ApiClient::new_with_base_url(tokens, Some(callback.clone()), mock_server.uri());

    // This should trigger token refresh and then succeed
    let result = api_client.get_machines().await;
    assert!(result.is_ok());

    // Verify callback was called
    let refreshed = callback.refreshed.lock().unwrap();
    assert!(*refreshed, "Token refresh callback should have been called");
}

#[tokio::test]
async fn test_api_client_token_refresh_failure_with_mock_server() {
    // Start a mock server
    let mock_server = MockServer::start().await;

    // Mock the token refresh endpoint to return failure
    Mock::given(method("POST"))
        .and(path("/auth/refreshtoken"))
        .respond_with(ResponseTemplate::new(401).set_body_string("Refresh token expired"))
        .mount(&mock_server)
        .await;

    // Create tokens with an expired access token
    let tokens = Credentials {
        access_token: "eyJhbGciOiJIUzUxMiJ9.invalid.expired".to_string(), // This will be considered expired
        refresh_token: "expired_refresh_token".to_string(),
        username: "test@example.com".to_string(),
    };

    // Create API client
    let mut api_client = ApiClient::new_with_base_url(tokens, None, mock_server.uri());

    // This should fail with refresh error
    let result = api_client.get_machines().await;
    assert!(result.is_err());

    let error_msg = result.unwrap_err().to_string();
    assert!(
        error_msg.contains("token refresh failed"),
        "Error should mention token refresh failure: {}",
        error_msg
    );
}
