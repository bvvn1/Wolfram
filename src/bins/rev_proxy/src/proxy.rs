use std::collections::BTreeSet;
use std::{collections::HashMap, sync::Arc};

use async_trait::async_trait;
use config::Config;

use pingora_core::Result;
use pingora_core::services::background::{GenBackgroundService, background_service};
use pingora_core::upstreams::peer::HttpPeer;
use pingora_load_balancing::discovery::Static;
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
            let lb = LoadBalancer::<RoundRobin>::from_backends(backends);

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
