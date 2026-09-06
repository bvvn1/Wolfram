use std::net::IpAddr;

use chrono::{DateTime, Utc};
use uuid::Uuid;

pub struct RefreshToken {
    id: Uuid,
    user_id: Uuid,
    token_hash: String,
    ip_address: Option<IpAddr>,
    user_agent: Option<String>,
    expires_at: DateTime<Utc>,
    revoked_at: Option<DateTime<Utc>>,
    replaced_by_token_id: Option<Uuid>,
    created_at: DateTime<Utc>,
}
