use std::collections::BTreeSet;
use std::time::Duration;
use std::{collections::HashMap, sync::Arc};

use async_trait::async_trait;
use config::Config;

use log::{error, info};
use pingora_core::Result;
use pingora_core::services::background::{GenBackgroundService, background_service};
use pingora_core::upstreams::peer::HttpPeer;
use pingora_load_balancing::discovery::Static;
use pingora_load_balancing::health_check::TcpHealthCheck;
use pingora_load_balancing::selection::RoundRobin;
use pingora_load_balancing::{Backend, Backends, LoadBalancer};
use pingora_proxy::{ProxyHttp, Session};

pub struct ServiceRuntime {
    pub name: String,
    pub lb: Arc<LoadBalancer<RoundRobin>>,
}

pub struct RevProxy {
    pub services: HashMap<String, Arc<ServiceRuntime>>,
}

impl RevProxy {
    pub fn init_from_config(
        config: &Config,
    ) -> anyhow::Result<(Self, Vec<GenBackgroundService<LoadBalancer<RoundRobin>>>)> {
        let mut services = HashMap::new();
        let mut background_services = Vec::new();

        for service in &config.services {
            let mut set = BTreeSet::new();
            for bckg in &service.backends {
                let addr = format!("{}:{}", bckg.host, bckg.port);
                let mut backend = Backend::new(addr.as_str())?;
                backend.weight = bckg.weight;
                set.insert(backend);
            }

            let backends = Backends::new(Static::new(set));
            let mut lb = LoadBalancer::<RoundRobin>::from_backends(backends);

            let hc = TcpHealthCheck::new();
            lb.set_health_check(hc);

            lb.health_check_frequency = Some(Duration::from_mins(1));

            let background = background_service(&service.name, lb);
            let lb_handle = background.task(); // Arc<LoadBalancer<RoundRobin>>

            services.insert(
                service.route.clone(),
                Arc::new(ServiceRuntime {
                    name: service.name.clone(),
                    lb: lb_handle,
                }),
            );

            background_services.push(background);
        }

        Ok((RevProxy { services }, background_services))
    }
}

pub struct RequestContext {
    pub service: Option<Arc<ServiceRuntime>>,
}

#[async_trait]
impl ProxyHttp for RevProxy {
    type CTX = RequestContext;

    fn new_ctx(&self) -> Self::CTX {
        RequestContext { service: None }
    }

    async fn request_filter(&self, session: &mut Session, ctx: &mut Self::CTX) -> Result<bool> {
        let host = session
            .req_header()
            .headers
            .get("host")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("")
            .to_string();

        match self.services.get(&host) {
            Some(service) => {
                ctx.service = Some(service.clone());
                Ok(false)
            }
            None => {
                session.respond_error(404).await?;
                Ok(true)
            }
        }
    }

    async fn logging(
        &self,
        session: &mut Session,
        e: Option<&pingora_error::Error>,
        _ctx: &mut Self::CTX,
    ) where
        Self::CTX: Send + Sync,
    {
        match e {
            Some(e) => {
                error!("{}", e.to_string());

                if e.cause.is_some() {
                    error!(
                        "cause of the error is: {} \n ",
                        e.cause.as_ref().unwrap().to_string()
                    )
                }
            }
            None => {
                info!(
                    "request forwarded from client with address: {:?} to {:?}",
                    session.client_addr().map(|a| {
                        a.to_string();
                    }),
                    session.server_addr().map(|a| a.to_string())
                )
            }
        }
    }

    async fn upstream_peer(
        &self,
        _session: &mut Session,
        ctx: &mut Self::CTX,
    ) -> Result<Box<HttpPeer>> {
        let service = ctx.service.as_ref().expect("service set in request_filter");

        let bkd = service
            .lb
            .select(b"", 256)
            .ok_or_else(|| pingora_core::Error::new_str("no healthy backend"))?;

        let peer = HttpPeer::new(bkd.addr, false, String::new());

        Ok(Box::new(peer))
    }
}
