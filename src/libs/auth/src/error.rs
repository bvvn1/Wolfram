use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Error during hashing of a password")]
    PasswordHashError(#[from] argon2::password_hash::Error),

    #[error("Error in database operations")]
    DatabaseError(#[from] sqlx::Error),

    #[error("Wrong username or password")]
    InvalidCredentials,

    #[error("Authentication Error: {0}")]
    AuthError(String),

    #[error("Context: ")]
    CustomContext(String),
}
