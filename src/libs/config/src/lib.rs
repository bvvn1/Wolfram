mod config;
mod error;
mod toml;
pub use self::config::Config;
pub use self::toml::load_config;

#[cfg(test)]
mod tests {
    use std::{path::PathBuf, str::FromStr};

    use super::*;

    #[test]
    fn test_parse() {
        let config = load_config(
            PathBuf::from_str("/home/lyubomir/Documents/Wolfram/config.toml").expect("padna"),
        )
        .unwrap();
        dbg!(config);
    }
}
