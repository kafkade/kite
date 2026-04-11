use thiserror::Error;

#[derive(Error, Debug)]
pub enum KiteError {
    #[error("Terminal error: {0}")]
    Terminal(#[from] std::io::Error),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Collector error: {0}")]
    Collector(String),
}
