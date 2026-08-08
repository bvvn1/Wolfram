mod backend;
mod health;

pub use backend::BackendPool;
pub use backend::init_from_config;

#[cfg(test)]
mod tests {
    use std::{path::PathBuf, str::FromStr};

    use config::load_config;

    use super::*;

    #[test]
    fn test_pool_init() {
        let config = load_config(PathBuf::from_str("../../../config.toml").unwrap()).unwrap();

        let pool = backend::init_from_config(&config);
        dbg!(pool);
    }
}
