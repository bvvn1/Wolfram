use serde::Deserialize;
use std::net::Ipv4Addr;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub services: Vec<ServiceConfig>,
}

#[derive(Debug, Deserialize)]
pub struct ServiceConfig {
    pub name: String,
    pub route: String,
    pub backends: Vec<BackendConfig>,
}

#[derive(Debug, Deserialize)]
pub struct BackendConfig {
    pub name: String,
    pub host: Ipv4Addr,
    pub port: u16,
    pub weight: usize,
}
