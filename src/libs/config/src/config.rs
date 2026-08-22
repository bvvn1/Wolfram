use serde::Deserialize;
use std::net::IpAddr;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub services: Vec<ServiceConfig>,
}

#[derive(Debug, Deserialize)]
pub struct ServiceConfig {
    pub name: String,
    pub host: String,
    pub prefix: String,
    pub backends: Vec<BackendConfig>,
}

#[derive(Debug, Deserialize)]
pub struct BackendConfig {
    pub name: String,
    pub host: IpAddr,
    pub port: u16,
    pub weight: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_multiple_services_and_backends() {
        let toml = r#"
            [[services]]
            name = "user-service"
            host = "api.local"
            prefix = "/user"

            [[services.backends]]
            name = "user-backend-1"
            host = "127.0.0.1"
            port = 3001
            weight = 1

            [[services.backends]]
            name = "user-backend-2"
            host = "127.0.0.1"
            port = 3002
            weight = 3

            [[services]]
            name = "auth-service"
            host = "api.local"
            prefix = "/auth"

            [[services.backends]]
            name = "auth-backend-1"
            host = "10.0.0.5"
            port = 4001
            weight = 2
        "#;

        let config: Config = toml::from_str(toml).unwrap();

        assert_eq!(config.services.len(), 2);

        let user = &config.services[0];
        assert_eq!(user.name, "user-service");
        assert_eq!(user.host, "api.local");
        assert_eq!(user.prefix, "/user");
        assert_eq!(user.backends.len(), 2);
        assert_eq!(user.backends[0].name, "user-backend-1");
        assert_eq!(
            user.backends[0].host,
            "127.0.0.1".parse::<IpAddr>().unwrap()
        );
        assert_eq!(user.backends[0].port, 3001);
        assert_eq!(user.backends[0].weight, 1);
        assert_eq!(user.backends[1].weight, 3);

        let auth = &config.services[1];
        assert_eq!(auth.name, "auth-service");
        assert_eq!(auth.backends[0].host, "10.0.0.5".parse::<IpAddr>().unwrap());
    }

    #[test]
    fn parses_ipv6_backend_host() {
        let toml = r#"
            [[services]]
            name = "v6-service"
            host = "api.local"
            prefix = "/v6"

            [[services.backends]]
            name = "v6-backend"
            host = "::1"
            port = 8080
            weight = 1
        "#;

        let config: Config = toml::from_str(toml).unwrap();
        let backend = &config.services[0].backends[0];

        assert_eq!(backend.host, "::1".parse::<IpAddr>().unwrap());
        assert_eq!(backend.port, 8080);
    }

    #[test]
    fn rejects_missing_required_field() {
        let toml = r#"
            [[services]]
            name = "incomplete-service"
            host = "api.local"

            [[services.backends]]
            name = "backend"
            host = "127.0.0.1"
            port = 3001
            weight = 1
        "#;

        let result = toml::from_str::<Config>(toml);
        assert!(result.is_err());
    }

    #[test]
    fn rejects_invalid_ip_address() {
        let toml = r#"
            [[services]]
            name = "bad-ip-service"
            host = "api.local"
            prefix = "/bad"

            [[services.backends]]
            name = "bad-ip-backend"
            host = "not-an-ip"
            port = 3001
            weight = 1
        "#;

        let result = toml::from_str::<Config>(toml);
        assert!(result.is_err());
    }
}
