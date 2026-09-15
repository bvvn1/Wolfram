use chrono::{DateTime, Duration, Utc};
use sha256::digest;
use sqlx::PgPool;
use uuid::Uuid;

use crate::crud::refresh_tokens::generate_random_token::generate_random_secure_token;
use crate::error::Error::{self, AuthError};
use crate::models::refresh_token::RefreshToken;

pub async fn rotate_refresh_token(current_token: &str, pool: &PgPool) -> Result<String, Error> {
    let current_token_hash = digest(current_token);

    let mut tx = pool.begin().await?;

    let old_token = sqlx::query_as!(
        RefreshToken,
        r#"
        SELECT * FROM refresh_tokens WHERE token_hash = $1 FOR UPDATE

        "#,
        &current_token_hash
    )
    .fetch_optional(&mut *tx)
    .await?
    .ok_or_else(|| {
        log::error!("Refresh Token not found");
        AuthError("Token Not Found".to_string())
    })?;

    if old_token.revoked_at.is_some() {
        sqlx::query!(
            r#"DELETE FROM refresh_tokens WHERE user_id = $1"#,
            old_token.user_id
        )
        .execute(&mut *tx)
        .await?;
        tx.commit().await?;
        return Err(AuthError(
            "Token Already revoked. (Token Theft)".to_string(),
        ));
    }

    if old_token.expires_at < Utc::now() {
        return Err(AuthError("Token Expired".to_string()));
    }

    let new_token_id = Uuid::new_v4();
    let new_token = generate_random_secure_token(32);
    let new_token_hash = digest(&new_token);

    sqlx::query!(
        r#"
            INSERT INTO refresh_tokens
            (id, user_id, token_hash, ip_address, user_agent, expires_at)
            VALUES ($1, $2, $3, $4, $5, $6)
        "#,
        &new_token_id,
        &old_token.user_id,
        new_token_hash,
        old_token.ip_address,
        old_token.user_agent,
        Utc::now() + Duration::days(7),
    )
    .execute(&mut *tx)
    .await?;

    let updated = sqlx::query!(
        r#"
        UPDATE refresh_tokens
        SET revoked_at = now(), replaced_by_token_id = $1
        WHERE id = $2 AND revoked_at IS NULL
        "#,
        new_token_id,
        &old_token.id
    )
    .execute(&mut *tx)
    .await?;

    if updated.rows_affected() == 0 {
        return Err(AuthError("Concurrent rotation detected".to_string()));
    }

    tx.commit().await?;

    Ok(new_token)
}
