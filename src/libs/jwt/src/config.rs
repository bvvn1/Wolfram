use std::time::Duration;

use jsonwebtoken::Algorithm;

pub const JWT_ALG: Algorithm = Algorithm::ES256;
pub const JWT_VALIDITY: Duration = Duration::from_hours(2);
