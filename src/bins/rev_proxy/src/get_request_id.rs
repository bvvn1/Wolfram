use pingora_proxy::Session;

use crate::proxy::RevProxy;

impl RevProxy {
    pub fn get_request_appid(session: &mut Session) -> Option<String> {
        match session.client_addr() {
            Some(v) => match v.as_inet() {
                Some(addr) => Some(addr.ip().to_string()),
                None => None,
            },
            None => None,
        }
    }
}
