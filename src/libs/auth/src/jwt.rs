use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JWTClaims {
    pub sub: Uuid,  // subject
    pub iat: usize, // issued at
    pub exp: usize, //expires
    pub roles: Vec<String>,
}
