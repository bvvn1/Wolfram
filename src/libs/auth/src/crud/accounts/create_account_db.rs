use argon2::{Argon2, Params, PasswordHasher};
use sqlx::PgPool;
pub async fn create_account_db(
    pool: &PgPool,
    username: &String,
    email: &String,
    password: &String,
    is_admin: bool,
) -> Result<(), crate::error::Error> {
    let id = uuid::Uuid::new_v4();

    let argon2 = Argon2::new(
        argon2::Algorithm::default(),
        argon2::Version::default(),
        Params::default(),
    );

    let hashed_password = argon2.hash_password(password.as_bytes())?.to_string();

    sqlx::query_as!(
        crate::models::user::User,
        r#"INSERT INTO users (id, email, password_hash, is_admin)
            VALUES ($1, $2, $3, $4)
            RETURNING id, email, password_hash, is_admin, created_at, updated_at"#,
        id,
        email,
        hashed_password,
        is_admin,
    )
    .fetch_one(pool)
    .await?;

    Ok(())
}
