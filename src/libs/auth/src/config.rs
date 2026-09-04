use jsonwebtoken::Algorithm;
use std::time::Duration;

pub const JWT_ALG: Algorithm = Algorithm::ES256;
pub const JWT_VALIDITY: Duration = Duration::from_hours(2);
