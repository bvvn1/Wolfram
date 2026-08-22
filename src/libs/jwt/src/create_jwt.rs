use jsonwebtoken::{EncodingKey, Header, encode};

use crate::{config::JWT_ALG, jwt::JWTClaims};

fn create_jwt(claims: JWTClaims, key: EncodingKey) -> anyhow::Result<String> {
    let token = encode(&Header::new(JWT_ALG), &claims, &key)?;
    Ok(token)
}
