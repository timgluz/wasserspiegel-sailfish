use std::fmt;

#[derive(Debug, Clone)]
pub enum CoreError {
    Config(String),
    Auth(String),
    NotFound(String),
    Server(u16, String),
    Network(String),
    Parse(String),
    Cache(String),
}

impl fmt::Display for CoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CoreError::Config(m) => write!(f, "configuration error: {m}"),
            CoreError::Auth(m) => write!(f, "authentication failed: {m}"),
            CoreError::NotFound(m) => write!(f, "not found: {m}"),
            CoreError::Server(code, m) => write!(f, "server error {code}: {m}"),
            CoreError::Network(m) => write!(f, "network error: {m}"),
            CoreError::Parse(m) => write!(f, "failed to parse response: {m}"),
            CoreError::Cache(m) => write!(f, "cache error: {m}"),
        }
    }
}

impl std::error::Error for CoreError {}

impl From<serde_json::Error> for CoreError {
    fn from(e: serde_json::Error) -> Self {
        CoreError::Parse(e.to_string())
    }
}

impl From<ureq::Error> for CoreError {
    fn from(e: ureq::Error) -> Self {
        match e {
            ureq::Error::StatusCode(code) => match code {
                401 | 403 => CoreError::Auth(e.to_string()),
                _ => CoreError::Server(code, e.to_string()),
            },
            other => CoreError::Network(other.to_string()),
        }
    }
}

pub type CoreResult<T> = Result<T, CoreError>;
