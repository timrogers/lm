//! Basic unit tests for the lm crate.

// Test that extract_jwt_expiry correctly parses the `exp` claim.
#[cfg(test)]
mod tests {
    use lm::LaMarzoccoClient;
    use jsonwebtoken::{encode, Header, EncodingKey};
    use serde::{Serialize, Deserialize};

    #[derive(Debug, Serialize, Deserialize)]
    struct TestClaims {
        exp: u64,
    }

    #[test]
    fn test_extract_jwt_expiry() {
        // Arrange: create a token that expires shortly in the future.
        let exp = (std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs())
            + 60; // +60 seconds
        let claims = TestClaims { exp };
        let token = encode(&Header::default(), &claims, &EncodingKey::from_secret(b""))
            .expect("failed to encode test token");

        // Act
        let parsed_exp = LaMarzoccoClient::extract_jwt_expiry(&token)
            .expect("failed to extract exp from token");

        // Assert
        assert_eq!(parsed_exp, exp);
    }

    #[test]
    fn test_extract_jwt_expiry_with_invalid_token() {
        // An obviously invalid JWT string should result in an error.
        let invalid_token = "not.a.valid.token";
        let result = LaMarzoccoClient::extract_jwt_expiry(invalid_token);
        assert!(result.is_err());
    }

    #[test]
    fn test_is_authenticated_flag() {
        let client = LaMarzoccoClient::new();
        assert!(!client.is_authenticated());
    }
}