use thiserror::Error;

#[derive(Error, Debug)]
pub enum PerceptionError {
    #[error("Camera error: {0}")]
    CameraError(String),
    
    #[error("Inference error: {0}")]
    InferenceError(String),
    
    #[error("Messaging error: {0}")]
    MessagingError(String),
    
    #[error("Configuration error: {0}")]
    ConfigError(String),
    
    #[error("Processing error: {0}")]
    ProcessingError(String),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("Serialization error: {0}")]
    SerializationError(String),
    
    #[error("Resource exhausted: {0}")]
    ResourceExhausted(String),
    
    #[error("Timeout: {0}")]
    Timeout(String),
    
    #[error("Unknown error: {0}")]
    Unknown(String),
}

impl From<ort::Error> for PerceptionError {
    fn from(error: ort::Error) -> Self {
        PerceptionError::InferenceError(error.to_string())
    }
}

impl From<zmq::Error> for PerceptionError {
    fn from(error: zmq::Error) -> Self {
        PerceptionError::MessagingError(error.to_string())
    }
}

impl From<gstreamer::Error> for PerceptionError {
    fn from(error: gstreamer::Error) -> Self {
        PerceptionError::CameraError(error.to_string())
    }
}

impl From<serde_json::Error> for PerceptionError {
    fn from(error: serde_json::Error) -> Self {
        PerceptionError::SerializationError(error.to_string())
    }
}

impl From<tokio::time::error::Elapsed> for PerceptionError {
    fn from(error: tokio::time::error::Elapsed) -> Self {
        PerceptionError::Timeout(error.to_string())
    }
}

pub type Result<T> = std::result::Result<T, PerceptionError>;