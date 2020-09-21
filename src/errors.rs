//! Error and Result types for the library 

/// Convenient Result type used by `elrond_rust`
pub type Result<T> = std::result::Result<T, ElrondClientError>;

/// Error type used by `elrond_rust`
#[derive(Debug, Clone)]
pub struct ElrondClientError{
    message: String
}

impl ElrondClientError {
    pub fn new(message: &str) -> Self {
        Self { message: message.to_string() }
    }
}

impl std::fmt::Display for ElrondClientError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}