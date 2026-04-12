use thiserror::Error;

#[derive(Error, Debug)]
#[allow(dead_code)]
pub enum KiteError {
    #[error("Terminal error: {0}")]
    Terminal(#[from] std::io::Error),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Collector error: {0}")]
    Collector(String),
}
