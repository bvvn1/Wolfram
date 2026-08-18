use anyhow::{Ok, Result};
use config::config::BackendConfig;
use pingora_load_balancing::Backend;

use crate::error::Error::{self, PrefixStrippingError};

pub trait ToBackend {
    fn to_backend(&self) -> Result<Backend>;
}

impl ToBackend for BackendConfig {
    fn to_backend(&self) -> Result<Backend> {
        let addr = std::net::SocketAddr::new(self.host, self.port).to_string();
        let mut backend = Backend::new(&addr)?;
        backend.weight = self.weight;
        Ok(backend)
    }
}

pub trait PrefixIndexable {
    fn strip_index_prefix(&self) -> std::result::Result<String, Error>;
}

impl PrefixIndexable for str {
    fn strip_index_prefix(&self) -> std::result::Result<String, Error> {
        self.split('/')
            .nth(1)
            .map(|s| format!("/{s}"))
            .ok_or(PrefixStrippingError())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::IpAddr;

    fn backend(host: &str, port: u16, weight: usize) -> BackendConfig {
        BackendConfig {
            name: "test".to_string(),
            host: host.parse::<IpAddr>().unwrap(),
            port,
            weight,
        }
    }

    #[test]
    fn converts_ipv4_address_and_weight() {
        let b = backend("127.0.0.1", 3001, 5).to_backend().unwrap();
        assert_eq!(b.addr.to_string(), "127.0.0.1:3001");
        assert_eq!(b.weight, 5);
    }

    #[test]
    fn converts_ipv6_address_and_weight() {
        let b = backend("::1", 8080, 2).to_backend().unwrap();
        assert_eq!(b.addr.to_string(), "[::1]:8080");
        assert_eq!(b.weight, 2);
    }
}
