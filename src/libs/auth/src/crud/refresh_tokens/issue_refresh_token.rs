use std::net::IpAddr;

use chrono::{Duration, Utc};
use sha256::digest;
use sqlx::PgPool;
use uuid::Uuid;

use crate::crud::refresh_tokens::generate_random_token::generate_random_secure_token;
use crate::error::Error;

pub async fn issue_refresh_token(
    user_id: Uuid,
    pool: &PgPool,
    ip: Option<IpAddr>,
    user_agent: Option<String>,
) -> Result<String, Error> {
    let raw_token = generate_random_secure_token(32);

    let token_hash = digest(&raw_token);

    let id = Uuid::new_v4();

    sqlx::query_as!(
        crate::models::refresh_token::RefreshToken,
        r#"
            INSERT INTO refresh_tokens
            (id, user_id, token_hash, ip_address, user_agent, expires_at)
            VALUES ($1, $2, $3, $4, $5, $6)
        "#,
        id,
        user_id,
        token_hash,
        ip.map(|i| i.to_string()),
        user_agent,
        Utc::now() + Duration::days(7),
    )
    .execute(pool)
    .await?;
    Ok(raw_token)
}
