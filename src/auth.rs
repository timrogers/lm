use anyhow::Result;
use log::debug;
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct LoginResponse {
    #[serde(rename = "accessToken")]
    pub access_token: String,
}

#[derive(Deserialize)]
pub struct ErrorResponse {
    #[allow(dead_code)]
    pub error: String,
    #[allow(dead_code)]
    pub message: Option<String>,
}

pub async fn authenticate_with_url(
    client: &reqwest::Client,
    base_url: &str,
    username: &str,
    password: &str,
) -> Result<String> {
    let login_request = LoginRequest {
        username: username.to_string(),
        password: password.to_string(),
    };

    let response = client
        .post(&format!("{}/auth/signin", base_url))
        .json(&login_request)
        .send()
        .await?;

    let status = response.status();
    let response_text = response.text().await?;

    if status.is_success() {
        match serde_json::from_str::<LoginResponse>(&response_text) {
            Ok(login_response) => {
                debug!("Authentication successful");
                Ok(login_response.access_token)
            }
            Err(e) => {
                debug!("Failed to parse login response: {}", e);
                Err(anyhow::anyhow!("Failed to parse authentication response"))
            }
        }
    } else {
        debug!("Authentication failed with status: {}", status);
        Err(anyhow::anyhow!("Authentication failed: {}", response_text))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_response_parsing() {
        // Test successful auth response
        let json = r#"{"accessToken":"eyJhbGciOiJIUzUxMiJ9.eyJzdWIiOiJtZUB0aW1yb2dlcnMuY28udWsiLCJpYXQiOjE3NDg1MTM0MDIsImV4cCI6MTc0ODUxNzAwMn0.fQJam2zsJopWMKtti0gOJ_1uUfyFop5tixsnlMWu-qhQeg0vb6BG8nTdRRx2Hw_ORxGLPrN4xyJatzpKPJ5YDA"}"#;

        let auth_response: LoginResponse = serde_json::from_str(json).unwrap();
        assert_eq!(
            auth_response.access_token,
            "eyJhbGciOiJIUzUxMiJ9.eyJzdWIiOiJtZUB0aW1yb2dlcnMuY28udWsiLCJpYXQiOjE3NDg1MTM0MDIsImV4cCI6MTc0ODUxNzAwMn0.fQJam2zsJopWMKtti0gOJ_1uUfyFop5tixsnlMWu-qhQeg0vb6BG8nTdRRx2Hw_ORxGLPrN4xyJatzpKPJ5YDA"
        );
    }

    #[test]
    fn test_login_request_serialization() {
        let request = LoginRequest {
            username: "test@example.com".to_string(),
            password: "password123".to_string(),
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("test@example.com"));
        assert!(json.contains("password123"));
    }
}
