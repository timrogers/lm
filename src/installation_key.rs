use anyhow::Result;
use base64::{engine::general_purpose::STANDARD, Engine as _};
use p256::{
    ecdsa::{signature::Signer, Signature, SigningKey},
    elliptic_curve::rand_core::OsRng,
    SecretKey,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

/// Installation key containing cryptographic material for authentication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallationKey {
    /// 32-byte secret for proof generation
    #[serde(
        serialize_with = "serialize_bytes_as_base64",
        deserialize_with = "deserialize_base64_as_bytes"
    )]
    pub secret: Vec<u8>,
    /// ECDSA private key on P-256 curve  
    #[serde(
        serialize_with = "serialize_signing_key_as_base64",
        deserialize_with = "deserialize_base64_as_signing_key"
    )]
    pub private_key: SigningKey,
    /// Installation ID (UUID)
    pub installation_id: String,
}

impl InstallationKey {
    /// Get the public key in base64-encoded DER format
    pub fn public_key_b64(&self) -> String {
        let verifying_key = *self.private_key.verifying_key();
        let public_key_bytes = verifying_key.to_encoded_point(false);
        STANDARD.encode(public_key_bytes.as_bytes())
    }

    /// Get the base string: installation_id.sha256(public_key_der_bytes)
    pub fn base_string(&self) -> String {
        let verifying_key = *self.private_key.verifying_key();
        let public_key_bytes = verifying_key.to_encoded_point(false);
        let mut hasher = Sha256::new();
        hasher.update(public_key_bytes.as_bytes());
        let pub_hash = hasher.finalize();
        let pub_hash_b64 = STANDARD.encode(pub_hash);
        format!("{}.{}", self.installation_id, pub_hash_b64)
    }
}

/// Generate installation key from installation ID following the Python pattern
pub fn generate_installation_key(installation_id: String) -> Result<InstallationKey> {
    // Generate ECDSA private key on P-256 curve
    let secret_key = SecretKey::random(&mut OsRng);
    let signing_key = SigningKey::from(secret_key);
    let verifying_key = *signing_key.verifying_key();

    // Get public key bytes in uncompressed format
    let public_key_bytes = verifying_key.to_encoded_point(false);
    let pub_b64 = STANDARD.encode(public_key_bytes.as_bytes());

    // Create installation hash  
    let mut hasher = Sha256::new();
    hasher.update(installation_id.as_bytes());
    let inst_hash = hasher.finalize();
    let inst_hash_b64 = STANDARD.encode(inst_hash);

    // Create triple: installation_id.pub_b64.inst_hash_b64
    let triple = format!("{}.{}.{}", installation_id, pub_b64, inst_hash_b64);

    // Generate 32-byte secret from triple
    let mut secret_hasher = Sha256::new();
    secret_hasher.update(triple.as_bytes());
    let secret_bytes = secret_hasher.finalize();

    Ok(InstallationKey {
        secret: secret_bytes.to_vec(),
        private_key: signing_key,
        installation_id,
    })
}

/// Generate a new random installation ID (UUID v4)
pub fn generate_installation_id() -> String {
    Uuid::new_v4().to_string().to_lowercase()
}

/// La Marzocco's custom proof generation algorithm (Y5.e equivalent)
pub fn generate_request_proof(base_string: &str, secret32: &[u8]) -> Result<String> {
    if secret32.len() != 32 {
        return Err(anyhow::anyhow!("secret must be 32 bytes"));
    }

    let mut work = secret32.to_vec(); // Make mutable copy

    for byte_val in base_string.as_bytes() {
        let idx = (*byte_val as usize) % 32;
        let shift_idx = (idx + 1) % 32;
        let shift_amount = work[shift_idx] & 7; // 0-7 bit shift

        // XOR then rotate left
        let xor_result = byte_val ^ work[idx];
        let rotated = if shift_amount == 0 {
            xor_result
        } else {
            (xor_result << shift_amount) | (xor_result >> (8 - shift_amount))
        };
        work[idx] = rotated;
    }

    let mut hasher = Sha256::new();
    hasher.update(&work);
    let result = hasher.finalize();
    Ok(STANDARD.encode(result))
}

/// Generate extra headers for normal API calls after authentication
pub fn generate_extra_request_headers(installation_key: &InstallationKey) -> Result<Vec<(String, String)>> {
    // Generate nonce and timestamp
    let nonce = Uuid::new_v4().to_string().to_lowercase();
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)?
        .as_millis()
        .to_string();

    // Create proof using Y5.e algorithm: installation_id.nonce.timestamp
    let proof_input = format!("{}.{}.{}", installation_key.installation_id, nonce, timestamp);
    let proof = generate_request_proof(&proof_input, &installation_key.secret)?;

    // Create signature data: installation_id.nonce.timestamp.proof
    let signature_data = format!("{}.{}", proof_input, proof);

    // Sign with ECDSA
    let signature: Signature = installation_key.private_key.sign(signature_data.as_bytes());
    let signature_b64 = STANDARD.encode(signature.to_der());

    // Return headers
    Ok(vec![
        ("X-App-Installation-Id".to_string(), installation_key.installation_id.clone()),
        ("X-Timestamp".to_string(), timestamp),
        ("X-Nonce".to_string(), nonce),
        ("X-Request-Signature".to_string(), signature_b64),
    ])
}

/// Serde helper functions for serialization
fn serialize_bytes_as_base64<S>(bytes: &Vec<u8>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let b64 = STANDARD.encode(bytes);
    serializer.serialize_str(&b64)
}

fn deserialize_base64_as_bytes<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    STANDARD
        .decode(s)
        .map_err(serde::de::Error::custom)
}

fn serialize_signing_key_as_base64<S>(signing_key: &SigningKey, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let bytes = signing_key.to_bytes();
    let b64 = STANDARD.encode(bytes);
    serializer.serialize_str(&b64)
}

fn deserialize_base64_as_signing_key<'de, D>(deserializer: D) -> Result<SigningKey, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    let bytes = STANDARD
        .decode(s)
        .map_err(serde::de::Error::custom)?;
    let secret_key = SecretKey::from_slice(&bytes)
        .map_err(serde::de::Error::custom)?;
    Ok(SigningKey::from(secret_key))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_installation_id_generation() {
        let id1 = generate_installation_id();
        let id2 = generate_installation_id();
        
        // IDs should be different
        assert_ne!(id1, id2);
        
        // IDs should be valid UUIDs (36 characters with dashes)
        assert_eq!(id1.len(), 36);
        assert_eq!(id2.len(), 36);
        assert!(id1.contains('-'));
        assert!(id2.contains('-'));
    }

    #[test]
    fn test_installation_key_generation() {
        let installation_id = "test-installation-id".to_string();
        let key = generate_installation_key(installation_id.clone()).unwrap();
        
        assert_eq!(key.installation_id, installation_id);
        assert_eq!(key.secret.len(), 32);
        
        // Test that we can get public key
        let pub_key_b64 = key.public_key_b64();
        assert!(!pub_key_b64.is_empty());
        
        // Test base string format
        let base_string = key.base_string();
        assert!(base_string.starts_with(&installation_id));
        assert!(base_string.contains('.'));
    }

    #[test]
    fn test_request_proof_generation() {
        let secret = vec![0u8; 32]; // All zeros for testing
        let base_string = "test.base.string";
        
        let proof = generate_request_proof(base_string, &secret).unwrap();
        assert!(!proof.is_empty());
        
        // Should be base64 encoded SHA256 hash (44 characters)
        assert_eq!(proof.len(), 44);
    }

    #[test]
    fn test_request_proof_error_on_wrong_secret_size() {
        let secret = vec![0u8; 31]; // Wrong size
        let base_string = "test";
        
        let result = generate_request_proof(base_string, &secret);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("32 bytes"));
    }

    #[test]
    fn test_extra_request_headers_generation() {
        let installation_id = "test-id".to_string();
        let key = generate_installation_key(installation_id.clone()).unwrap();
        
        let headers = generate_extra_request_headers(&key).unwrap();
        
        // Should have 4 headers
        assert_eq!(headers.len(), 4);
        
        // Check header names
        let header_names: Vec<String> = headers.iter().map(|(k, _)| k.clone()).collect();
        assert!(header_names.contains(&"X-App-Installation-Id".to_string()));
        assert!(header_names.contains(&"X-Timestamp".to_string()));
        assert!(header_names.contains(&"X-Nonce".to_string()));
        assert!(header_names.contains(&"X-Request-Signature".to_string()));
        
        // Check installation ID matches
        let installation_id_header = headers.iter()
            .find(|(k, _)| k == "X-App-Installation-Id")
            .unwrap();
        assert_eq!(installation_id_header.1, installation_id);
    }

    #[test]
    fn test_installation_key_serialization() {
        let installation_id = "test-id".to_string();
        let key = generate_installation_key(installation_id.clone()).unwrap();
        
        // Test JSON serialization
        let json = serde_json::to_string(&key).unwrap();
        assert!(!json.is_empty());
        
        // Test deserialization
        let deserialized: InstallationKey = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.installation_id, key.installation_id);
        assert_eq!(deserialized.secret, key.secret);
        
        // Verify keys work the same
        assert_eq!(deserialized.public_key_b64(), key.public_key_b64());
        assert_eq!(deserialized.base_string(), key.base_string());
    }
}