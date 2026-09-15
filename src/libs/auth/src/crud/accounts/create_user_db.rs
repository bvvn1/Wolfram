use argon2::{Argon2, Params, PasswordHasher};
use sqlx::PgPool;
use uuid::Uuid;

pub async fn create_user_db(
    pool: &PgPool,
    username: &str,
    email: Option<&String>,
    password: &str,
    is_admin: bool,
) -> Result<Uuid, crate::error::Error> {
    let id = uuid::Uuid::new_v4();

    let argon2 = Argon2::default();

    let hashed_password = argon2.hash_password(password.as_bytes())?.to_string();

    let user = sqlx::query_as!(
        crate::models::user::User,
        r#"INSERT INTO users (id, username, email, password_hash, is_admin)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id, username, email, password_hash, is_admin, created_at, updated_at"#,
        id,
        username,
        email,
        hashed_password,
        is_admin,
    )
    .fetch_one(pool)
    .await?;

    Ok(user.id)
}
