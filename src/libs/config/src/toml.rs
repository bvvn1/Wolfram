use super::error::Error;
use crate::config::Config;
use std::path::PathBuf;

pub fn load_config(location: PathBuf) -> Result<Config, Error> {
    let toml_string = std::fs::read_to_string(location)?;
    let config = toml::from_str::<Config>(&toml_string)?;
    Ok(config)
}
