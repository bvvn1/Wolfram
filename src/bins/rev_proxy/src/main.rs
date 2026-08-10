pub mod proxy;

use std::{path::PathBuf, str::FromStr};

use anyhow::Error;

use config::load_config;
use pingora_core::server::Server;

use crate::proxy::RevProxy;

fn main() -> anyhow::Result<(), Error> {
    env_logger::init();

    let config = load_config(PathBuf::from_str("../../../../config.toml").unwrap())?;

    let (rev_proxy, bkgservices) = RevProxy::init_from_config(&config)?;

    let mut server = Server::new(None)?;
    server.bootstrap();
    for srv in bkgservices {
        server.add_service(srv);
    }

    let mut proxy = pingora_proxy::http_proxy_service(&server.configuration, rev_proxy);

    proxy.add_tcp("0.0.0.0:6193");

    server.add_service(proxy);

    server.run_forever();
}
