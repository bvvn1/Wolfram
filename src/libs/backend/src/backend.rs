use std::{
    net::Ipv4Addr,
    sync::{Arc, atomic::AtomicU64},
};

use arc_swap::ArcSwap;
use config::Config;
use dashmap::DashSet;

use crate::health::Health::{self, Unknown};

#[derive(Debug)]
pub struct BackendPool(ArcSwap<Vec<Arc<BackendRuntime>>>);

#[derive(Debug)]
pub struct BackendRuntime {
    pub name: String,
    pub host: Ipv4Addr,
    pub port: u16,
    pub weight: u64,
    pub current_requests: AtomicU64,
    pub health: Health,
}

pub fn init_from_config(cfg: &Config) -> BackendPool {
    let mut v: Vec<Arc<BackendRuntime>> = Vec::new();

    for b in &cfg.backends {
        let temp_backend = BackendRuntime {
            name: b.name.to_owned(),
            host: b.host,
            port: b.port,
            weight: b.weight,
            current_requests: AtomicU64::new(0),
            health: Unknown,
        };

        v.push(Arc::new(temp_backend));
    }

    BackendPool {
        0: ArcSwap::from_pointee(v),
    }
}
