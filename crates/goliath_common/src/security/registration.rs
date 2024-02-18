use crate::core::NodeType;
use crate::dev::NaiveDb;
use base64::{DecodeError, Engine};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::Mutex;

#[derive(Clone, Serialize, Deserialize)]
pub struct RegistrationRequest {
    pub id: String,
    pub timestamp: u128,
    pub hash: String,
    pub node_type: NodeType,
}

#[derive(Clone, Debug, Error)]
pub enum RegistrationError {
    #[error("No such client found")]
    ClientNotFound,
    #[error("No such vehicle found")]
    VehicleNotFound,
    #[error("Node type unrecognized")]
    UnknownNodeType,
    #[error("Registration failed: mismatching hash")]
    MismatchedHash,
}

impl RegistrationRequest {
    #[cfg(feature = "development")]
    pub fn verify_hash<DB: NaiveDb>(
        &self,
        cache_db: Arc<Mutex<DB>>,
    ) -> Result<NodeType, Box<dyn Error>> {
        let secret_key = match self.node_type {
            NodeType::Client => cache_db
                .blocking_lock()
                .get_client(&self.id)
                .map(|client| client.secret_key.clone())
                .ok_or(RegistrationError::ClientNotFound),
            NodeType::Vehicle => cache_db
                .blocking_lock()
                .get_client(&self.id)
                .map(|client| client.secret_key.clone())
                .ok_or(RegistrationError::VehicleNotFound),
            _ => Err(RegistrationError::UnknownNodeType),
        }?;

        let generated_hash = generate_registration_hash(&self.id, self.timestamp, &secret_key)?;
        if generated_hash == self.hash {
            Ok(self.node_type)
        } else {
            Err(RegistrationError::MismatchedHash.into())
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct RegistrationResponse {
    pub status: String,
    pub msg: String,
}

pub fn generate_registration_hash(
    id: &str,
    timestamp: u128,
    key: &str,
) -> Result<String, DecodeError> {
    let mut out = String::with_capacity(1024);
    base64::engine::general_purpose::STANDARD
        .decode(key)
        .map(|res| {
            let hash = sha256::digest(format!("{id}{timestamp}{}", String::from_utf8_lossy(&res)));
            base64::engine::general_purpose::STANDARD.encode_string(&hash, &mut out);
            out
        })
}
