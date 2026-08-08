use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("I/O error")]
    IoError(#[from] std::io::Error),
    #[error("Error parsing toml")]
    TomlParseError(#[from] toml::de::Error),
}
