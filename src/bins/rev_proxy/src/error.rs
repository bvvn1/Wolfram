use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Error when stripping prefix")]
    PrefixStrippingError(),
}
