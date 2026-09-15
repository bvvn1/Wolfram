use crate::error::Error;
use sqlx::PgPool;

pub async fn does_user_exists_db(username: &str, pool: &PgPool) -> Result<bool, Error> {
    let exists = sqlx::query_scalar!(
        "SELECT EXISTS(SELECT 1 FROM users WHERE username = $1)",
        username
    )
    .fetch_one(pool)
    .await?
    .unwrap_or(false);

    Ok(exists)
}
