use pingora_proxy::Session;

use crate::proxy::RevProxy;

impl RevProxy {
    pub fn get_request_appid(session: &mut Session) -> Option<String> {
        match session
            .req_header()
            .headers
            .get("appid")
            .map(|v| v.to_str())
        {
            Some(v) => match v {
                Ok(v) => Some(v.to_owned()),
                Err(_) => None,
            },
            None => None,
        }
    }
}
