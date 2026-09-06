-- Add migration script here
CREATE TABLE users IF NOT EXISTS (
    id UUID PRIMARY KEY,
    email TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    created_at TIMESTAMPZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_users_email ON users(email);
