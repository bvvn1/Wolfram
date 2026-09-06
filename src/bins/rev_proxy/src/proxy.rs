use std::collections::BTreeSet;
use std::time::Duration;
use std::{collections::HashMap, sync::Arc};

use async_trait::async_trait;
use bytes::Bytes;
use config::Config;

use log::{error, info};
use pingora_core::Result;
use pingora_core::services::background::{GenBackgroundService, background_service};
use pingora_core::upstreams::peer::HttpPeer;
use pingora_http::ResponseHeader;
use pingora_load_balancing::discovery::Static;
use pingora_load_balancing::health_check::TcpHealthCheck;
use pingora_load_balancing::selection::RoundRobin;
use pingora_load_balancing::{Backend, Backends, LoadBalancer};
use pingora_proxy::{ProxyHttp, Session};
use uuid::Uuid;

use crate::rate_limiter::{MAX_REQ_PER_SEC, RATE_LIMITER};
use crate::traits::{PrefixIndexable, ToBackend};

pub struct ServiceRuntime {
    pub name: String,
    pub lb: Arc<LoadBalancer<RoundRobin>>,
}

pub struct RevProxy {
    pub services: HashMap<String, Vec<(String, Arc<ServiceRuntime>)>>,
}

impl RevProxy {
    pub fn init_from_config(
        config: &Config,
    ) -> anyhow::Result<(Self, Vec<GenBackgroundService<LoadBalancer<RoundRobin>>>)> {
        let mut services = HashMap::new();
        let mut background_services = Vec::new();

        for service in &config.services {
            // let router = services
            //     .entry(service.host.to_owned())
            //     .or_insert_with(matchit::Router::new);
            let prefix_routes = services
                .entry(service.host.to_owned())
                .or_insert_with(Vec::new);

            let set: BTreeSet<Backend> = service
                .backends
                .iter()
                .map(|b| b.to_backend())
                .collect::<anyhow::Result<_>>()?;

            let backends = Backends::new(Static::new(set));

            let mut lb = LoadBalancer::<RoundRobin>::from_backends(backends);

            let hc = TcpHealthCheck::new();
            lb.set_health_check(hc);
            lb.health_check_frequency = Some(Duration::from_mins(1));
            let background = background_service(&service.name, lb);

            let lb_handle = background.task(); // Arc<LoadBalancer<RoundRobin>>
            background_services.push(background);

            prefix_routes.push((
                service.prefix.clone(),
                Arc::new(ServiceRuntime {
                    name: service.name.clone(),
                    lb: lb_handle,
                }),
            ));
        }

        for vec in services.values_mut() {
            vec.sort_by(|a, b| a.0.cmp(&b.0));
        }
        Ok((RevProxy { services }, background_services))
    }
}

pub struct RequestContext {
    pub service: Option<Arc<ServiceRuntime>>,
    pub request_id: Option<Uuid>,
    pub backend_addr: Option<String>,
}

#[async_trait]
impl ProxyHttp for RevProxy {
    type CTX = RequestContext;

    fn new_ctx(&self) -> Self::CTX {
        RequestContext {
            service: None,
            request_id: Some(Uuid::new_v4()),
            backend_addr: None,
        }
    }

    async fn request_filter(&self, session: &mut Session, ctx: &mut Self::CTX) -> Result<bool> {
        // RATE LIMIT -----------------------------
        match Self::get_request_appid(session) {
            Some(id) => {
                info!("{}", &id);
                let curr_window_requests = RATE_LIMITER.observe(&id, 1);
                if curr_window_requests > MAX_REQ_PER_SEC {
                    let mut header = ResponseHeader::build(429, None)?;
                    header.insert_header("X-Rate-Limit-Limit", MAX_REQ_PER_SEC.to_string())?;
                    header.insert_header("X-Rate-Limit-Remaining", "0")?;
                    header.insert_header("X-Rate-Limit-Reset", "1")?;
                    session.set_keepalive(None);
                    session
                        .write_response_header(Box::new(header), true)
                        .await?;
                    return Ok(true);
                }
            }
            None => (),
        };
        // -----------------------------------------------------------
        #[cfg(debug_assertions)]
        for (name, value) in session.req_header().headers.iter() {
            eprintln!("HEADER: {}: {:?}", name, value);
        }

        // HOST ROUTING ----------------
        let host = session
            .req_header()
            .headers
            .get("host")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("")
            .to_string();

        // --------------- prefix routing
        let Ok(path) = session.req_header().uri.path().strip_index_prefix() else {
            session
                .respond_error_with_body(
                    404,
                    Bytes::from(
                        "error during prefix routing: Error during the stripping of the prefix",
                    ),
                )
                .await?;
            return Ok(true);
        };

        #[cfg(debug_assertions)]
        dbg!(&path);

        match self.services.get(&host) {
            Some(router) => {
                let matched_prefix = router.iter().find(|r| path.starts_with(&r.0));

                match matched_prefix {
                    Some(a) => {
                        ctx.service = Some(a.1.clone());
                        Ok(false)
                    }
                    None => {
                        session
                            .respond_error_with_body(
                                404,
                                Bytes::from("error during prefix routing"),
                            )
                            .await?;
                        return Ok(true);
                    }
                }
            }
            None => {
                session
                    .respond_error_with_body(404, Bytes::from("error during host routing"))
                    .await?;
                Ok(true)
            }
        }
    }

    async fn logging(
        &self,
        session: &mut Session,
        e: Option<&pingora_error::Error>,
        ctx: &mut Self::CTX,
    ) where
        Self::CTX: Send + Sync,
    {
        match e {
            Some(e) => {
                error!("{}", e.to_string());

                if e.cause.is_some() {
                    error!("cause of the error is: {} \n ", e.cause.as_ref().unwrap())
                }
            }
            None => {
                info!(
                    "request with id: {:?} forwarded from client with address: {:?} to {:?}",
                    ctx.request_id,
                    session.client_addr().map(|a| { a.to_string() }),
                    ctx.backend_addr
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

        ctx.backend_addr = Some(bkd.addr.clone().to_string());

        let peer = HttpPeer::new(bkd.addr, false, String::new());

        Ok(Box::new(peer))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use config::config::{BackendConfig, Config, ServiceConfig};
    use std::net::IpAddr;

    fn backend(name: &str, host: &str, port: u16, weight: usize) -> BackendConfig {
        BackendConfig {
            name: name.to_string(),
            host: host.parse::<IpAddr>().unwrap(),
            port,
            weight,
        }
    }

    fn service(
        name: &str,
        host: &str,
        prefix: &str,
        backends: Vec<BackendConfig>,
    ) -> ServiceConfig {
        ServiceConfig {
            name: name.to_string(),
            host: host.to_string(),
            prefix: prefix.to_string(),
            backends,
        }
    }

    fn config(services: Vec<ServiceConfig>) -> Config {
        Config { services }
    }

    #[test]
    fn builds_host_and_prefix_routing_table() {
        let cfg = config(vec![
            service(
                "user-service",
                "api.local",
                "/user",
                vec![
                    backend("user-backend-1", "127.0.0.1", 3001, 1),
                    backend("user-backend-2", "127.0.0.1", 3002, 1),
                ],
            ),
            service(
                "auth-service",
                "api.local",
                "/auth",
                vec![backend("auth-backend-1", "127.0.0.1", 4001, 1)],
            ),
            service(
                "admin-user-service",
                "admin.local",
                "/user",
                vec![backend("admin-user-backend", "127.0.0.1", 5001, 1)],
            ),
        ]);

        let (rev_proxy, background_services) = RevProxy::init_from_config(&cfg).unwrap();

        // One router per distinct host.
        assert_eq!(rev_proxy.services.len(), 2);
        assert!(rev_proxy.services.contains_key("api.local"));
        assert!(rev_proxy.services.contains_key("admin.local"));

        // One background health-check service per configured service.
        assert_eq!(background_services.len(), 3);

        // api.local serves both /user and /auth prefixes.
        let api_router = &rev_proxy.services["api.local"];

        for vec in api_router {
            dbg!(&vec.0);
        }

        assert!(
            api_router
                .iter()
                .find(|r| r
                    .0
                    .starts_with("/user/123".strip_index_prefix().unwrap().as_str()))
                .is_some()
        );

        assert!(
            api_router
                .iter()
                .find(|r| r
                    .0
                    .starts_with("/auth/login".strip_index_prefix().unwrap().as_str()))
                .is_some()
        );

        let admin_router = &rev_proxy.services["admin.local"];

        assert!(
            admin_router
                .iter()
                .find(|r| r
                    .0
                    .starts_with("/user/456".strip_index_prefix().unwrap().as_str()))
                .is_some()
        );

        assert!(
            admin_router
                .iter()
                .find(|r| r
                    .0
                    .starts_with("/auth/login".strip_index_prefix().unwrap().as_str()))
                .is_none()
        );
    }

    #[test]
    fn unknown_prefix_has_no_match() {
        let cfg = config(vec![service(
            "user-service",
            "api.local",
            "/user",
            vec![backend("user-backend-1", "127.0.0.1", 3001, 1)],
        )]);

        let (rev_proxy, _) = RevProxy::init_from_config(&cfg).unwrap();

        let router = &rev_proxy.services["api.local"];

        assert!(
            router
                .iter()
                .find(|r| r
                    .0
                    .starts_with("/user/999".strip_index_prefix().unwrap().as_str()))
                .is_some()
        );

        assert!(
            router
                .iter()
                .find(|r| r
                    .0
                    .starts_with("/unknown".strip_index_prefix().unwrap().as_str()))
                .is_none()
        );
    }

    #[test]
    fn empty_services_yields_empty_routing_table() {
        let cfg = config(vec![]);

        let (rev_proxy, background_services) = RevProxy::init_from_config(&cfg).unwrap();

        assert!(rev_proxy.services.is_empty());
        assert!(background_services.is_empty());
    }
}
