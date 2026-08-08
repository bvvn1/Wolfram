use std::net::Ipv4Addr;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Backend {
    pub name: String,
    pub host: Ipv4Addr,
    pub port: u16,
    pub weight: u64,
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub backends: Vec<Backend>,
}
