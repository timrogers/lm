use lm::LaMarzoccoClient;
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
    use lm::types::{Machine, MachineStatus};
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
