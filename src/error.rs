use thiserror::Error;

/// Application-specific error types
#[derive(Error, Debug)]
pub enum DroasError {
    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Discord API error: {0}")]
    Discord(String),

    #[error("Database error: {0}")]
    Database(String),

    #[error("Image processing error: {0}")]
    ImageProcessing(String),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Network error: {0}")]
    Network(String),

    #[error("Validation error: {0}")]
    Validation(String),
}

impl DroasError {
    /// Create a configuration error
    pub fn config<S: Into<String>>(msg: S) -> Self {
        Self::Config(msg.into())
    }

    /// Create a Discord API error
    pub fn discord<S: Into<String>>(msg: S) -> Self {
        Self::Discord(msg.into())
    }

    /// Create a database error
    pub fn database<S: Into<String>>(msg: S) -> Self {
        Self::Database(msg.into())
    }

    /// Create an image processing error
    pub fn image_processing<S: Into<String>>(msg: S) -> Self {
        Self::ImageProcessing(msg.into())
    }

    /// Create a network error
    pub fn network<S: Into<String>>(msg: S) -> Self {
        Self::Network(msg.into())
    }

    /// Create a validation error
    pub fn validation<S: Into<String>>(msg: S) -> Self {
        Self::Validation(msg.into())
    }
}

/// Type alias for Results using our error type
pub type DroasResult<T> = Result<T, DroasError>;

/// Trait for graceful error handling
pub trait GracefulError {
    /// Handle error gracefully and log appropriately
    fn handle_gracefully(&self);
}

impl GracefulError for DroasError {
    fn handle_gracefully(&self) {
        match self {
            DroasError::Config(msg) => {
                tracing::error!(
                    "Configuration error (check your environment variables): {}",
                    msg
                );
            }
            DroasError::Discord(msg) => {
                tracing::error!(
                    "Discord API error (check your bot token and permissions): {}",
                    msg
                );
            }
            DroasError::Database(msg) => {
                tracing::error!("Database error (check database connectivity): {}", msg);
            }
            DroasError::ImageProcessing(msg) => {
                tracing::warn!("Image processing error (will retry): {}", msg);
            }
            DroasError::Network(msg) => {
                tracing::warn!("Network error (will retry): {}", msg);
            }
            DroasError::Io(err) => {
                tracing::error!("I/O error: {}", err);
            }
            DroasError::Validation(msg) => {
                tracing::error!("Validation error: {}", msg);
            }
        }
    }
}
