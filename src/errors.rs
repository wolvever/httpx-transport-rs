use pyo3::{exceptions::*, prelude::*};
use reqwest_middleware::Error as MiddlewareError;

/// Custom error types for the transport
#[derive(Debug, Clone, thiserror::Error)]
pub enum TransportError {
    #[error("Request timeout: {0}")]
    RequestTimeout(String),
    
    #[error("Connect timeout: {0}")]
    ConnectTimeout(String),
    
    #[error("Read timeout: {0}")]
    ReadTimeout(String),
    
    #[error("Connection error: {0}")]
    ConnectError(String),
    
    #[error("Read error: {0}")]
    ReadError(String),
    
    #[error("Write error: {0}")]
    WriteError(String),
    
    #[error("Pool timeout: {0}")]
    PoolTimeout(String),
    
    #[error("SSL error: {0}")]
    SSLError(String),
    
    #[error("Proxy error: {0}")]
    ProxyError(String),
    
    #[error("Local protocol error: {0}")]
    LocalProtocolError(String),
    
    #[error("Remote protocol error: {0}")]
    RemoteProtocolError(String),
    
    #[error("Invalid URL: {0}")]
    InvalidURL(String),
    
    #[error("Too many redirects")]
    TooManyRedirects,
    
    #[error("Other error: {0}")]
    Other(String),
}

impl From<reqwest::Error> for TransportError {
    fn from(err: reqwest::Error) -> Self {
        if err.is_timeout() {
            if err.is_connect() {
                TransportError::ConnectTimeout(err.to_string())
            } else {
                TransportError::RequestTimeout(err.to_string())
            }
        } else if err.is_connect() {
            TransportError::ConnectError(err.to_string())
        } else if err.is_redirect() {
            TransportError::TooManyRedirects
        } else if err.is_request() {
            TransportError::LocalProtocolError(err.to_string())
        } else {
            TransportError::Other(err.to_string())
        }
    }
}

impl From<MiddlewareError> for TransportError {
    fn from(err: MiddlewareError) -> Self {
        match err {
            MiddlewareError::Middleware(e) => TransportError::Other(e.to_string()),
            MiddlewareError::Reqwest(e) => e.into(),
        }
    }
}

impl From<TransportError> for PyErr {
    fn from(err: TransportError) -> Self {
        match err {
            TransportError::RequestTimeout(msg) => {
                PyErr::new::<PyTimeoutError, _>(format!("Request timeout: {}", msg))
            }
            TransportError::ConnectTimeout(msg) => {
                PyErr::new::<PyConnectionError, _>(format!("Connect timeout: {}", msg))
            }
            TransportError::ReadTimeout(msg) => {
                PyErr::new::<PyTimeoutError, _>(format!("Read timeout: {}", msg))
            }
            TransportError::ConnectError(msg) => {
                PyErr::new::<PyConnectionError, _>(format!("Connect error: {}", msg))
            }
            TransportError::ReadError(msg) => {
                PyErr::new::<PyIOError, _>(format!("Read error: {}", msg))
            }
            TransportError::WriteError(msg) => {
                PyErr::new::<PyIOError, _>(format!("Write error: {}", msg))
            }
            TransportError::PoolTimeout(msg) => {
                PyErr::new::<PyTimeoutError, _>(format!("Pool timeout: {}", msg))
            }
            TransportError::SSLError(msg) => {
                PyErr::new::<PyConnectionError, _>(format!("SSL error: {}", msg))
            }
            TransportError::ProxyError(msg) => {
                PyErr::new::<PyConnectionError, _>(format!("Proxy error: {}", msg))
            }
            TransportError::LocalProtocolError(msg) => {
                PyErr::new::<PyValueError, _>(format!("Local protocol error: {}", msg))
            }
            TransportError::RemoteProtocolError(msg) => {
                PyErr::new::<PyValueError, _>(format!("Remote protocol error: {}", msg))
            }
            TransportError::InvalidURL(msg) => {
                PyErr::new::<PyValueError, _>(format!("Invalid URL: {}", msg))
            }
            TransportError::TooManyRedirects => {
                PyErr::new::<PyValueError, _>("Too many redirects")
            }
            TransportError::Other(msg) => {
                PyErr::new::<PyRuntimeError, _>(format!("HTTP error: {}", msg))
            }
        }
    }
}

/// Result type for transport operations
pub type TransportResult<T> = Result<T, TransportError>; 