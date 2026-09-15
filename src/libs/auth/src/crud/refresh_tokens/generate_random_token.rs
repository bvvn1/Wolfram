use base64::{Engine, engine::general_purpose};
use rand::Rng;

pub(crate) fn generate_random_secure_token(byte_len: usize) -> String {
    let mut bytes = vec![0u8; byte_len];
    rand::rng().fill_bytes(&mut bytes);
    general_purpose::URL_SAFE_NO_PAD.encode(bytes)
}
