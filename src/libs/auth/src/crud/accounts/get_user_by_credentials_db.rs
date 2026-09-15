use std::time::Duration;

use crate::{error::Error, models::user::User};
use argon2::{Argon2, PasswordHash, PasswordVerifier};
use sqlx::PgPool;

pub async fn get_user_by_credentials_db(
    username: &str,
    password: &str,
    pool: &PgPool,
) -> Result<User, Error> {
    let Some(account) = sqlx::query_as!(
        User,
        r#"
        SELECT * FROM users WHERE username = $1

        "#,
        username
    )
    .fetch_optional(pool)
    .await?
    else {
        tokio::time::sleep(Duration::from_millis(80)).await;
        return Err(Error::InvalidCredentials);
    };

    let parsed_hash = PasswordHash::new(&account.password_hash)
        .map_err(|e| Error::CustomContext(e.to_string()))?;

    let argon2 = Argon2::default();

    match argon2.verify_password(password.as_bytes(), &parsed_hash) {
        Ok(_) => (),
        Err(_) => return Err(Error::InvalidCredentials),
    }

    Ok(account)
}
