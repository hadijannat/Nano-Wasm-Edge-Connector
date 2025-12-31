//! Shared types for Nano-Wasm Edge Connector
//! 
//! Common structures used by both host runtime and guest policy modules.

use serde::{Deserialize, Serialize};

/// Policy evaluation request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyRequest {
    /// Role of the requester (e.g., "admin", "operator", "viewer")
    #[serde(default)]
    pub role: Option<String>,
    
    /// Resource being accessed
    #[serde(default)]
    pub resource: Option<String>,
    
    /// Action being performed
    #[serde(default)]
    pub action: Option<String>,
    
    /// Whether the request is explicitly blocked
    #[serde(default)]
    pub blocked: bool,
}

/// Policy evaluation result
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PolicyResult {
    Allow,
    Deny,
}

impl PolicyResult {
    pub fn is_allowed(&self) -> bool {
        matches!(self, PolicyResult::Allow)
    }
}

/// Response from policy evaluation endpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyResponse {
    pub allowed: bool,
    pub policy_version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}
