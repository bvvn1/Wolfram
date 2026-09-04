use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JWTClaims {
    pub sub: Uuid,
    pub iat: usize,
    pub exp: usize,
    pub roles: Vec<String>,
}
