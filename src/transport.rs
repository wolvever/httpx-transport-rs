use std::sync::Arc;

use pyo3::prelude::*;
use pyo3::types::PyBytes;
use reqwest_middleware::ClientWithMiddleware;

use crate::client::get_client;
use crate::errors::TransportError;
use crate::streaming::{ByteStream, SyncByteStream, extract_body_from_python};
use crate::utils::{
    extract_method, extract_url, extract_headers, extract_extensions,
    create_response_object, extract_timeout_from_extensions, is_streaming_requested,
};

/// Async transport for httpx using Rust reqwest + tower
#[pyclass]
pub struct AsyncTransport {
    client: Arc<ClientWithMiddleware>,
}

#[pymethods]
impl AsyncTransport {
    #[new]
    fn new() -> Self {
        // Initialize tracing on first use
        crate::client::init_tracing();
        
        Self {
            client: get_client(),
        }
    }
    
    /// Handle an async HTTP request
    fn handle_async_request<'py>(
        &self,
        py: Python<'py>,
        request: &PyAny,
    ) -> PyResult<&'py PyAny> {
        let client = self.client.clone();
        
        // Extract request components while holding GIL
        let method = extract_method(request.getattr("method")?)?;
        let url = extract_url(request.getattr("url")?)?;
        let headers = extract_headers(request.getattr("headers")?)?;
        let extensions = extract_extensions(request.getattr("extensions")?)?;
        
        // Extract body
        let body = if let Ok(py_body) = request.getattr("content") {
            extract_body_from_python(py_body)?
        } else {
            reqwest::Body::from("")
        };
        
        // Check configuration from extensions
        let timeout = extract_timeout_from_extensions(&extensions);
        let streaming = is_streaming_requested(&extensions);
        
        // Release GIL and perform the request
        pyo3_asyncio::tokio::future_into_py(py, async move {
            let mut req_builder = client.request(method, url)
                .headers(headers)
                .body(body);
            
            // Apply timeout if specified
            if let Some(timeout_duration) = timeout {
                req_builder = req_builder.timeout(timeout_duration);
            }
            
            // Execute the request
            let response = req_builder.send().await
                .map_err(TransportError::from)?;
            
            // Extract response components
            let status = response.status().as_u16();
            let response_headers = response.headers().clone();
            let response_extensions = Some(extensions.clone());
            
            if streaming {
                // Create streaming response
                let stream = ByteStream::from_response(response);
                Python::with_gil(|py| {
                    let py_stream = Py::new(py, stream)?;
                    create_response_object(
                        py,
                        status,
                        response_headers,
                        None,  // No content for streaming
                        Some(py_stream.to_object(py)),
                        response_extensions,
                    )
                })
            } else {
                // Read full response body
                let bytes = response.bytes().await
                    .map_err(TransportError::from)?;
                
                Python::with_gil(|py| {
                    let py_content = PyBytes::new(py, &bytes);
                    create_response_object(
                        py,
                        status,
                        response_headers,
                        Some(py_content.into()),
                        None,  // No stream for non-streaming
                        response_extensions,
                    )
                })
            }
        })
    }
    
    /// Close the transport (cleanup)
    fn aclose<'py>(&self, py: Python<'py>) -> PyResult<&'py PyAny> {
        // For now, we don't need to do anything as the client is shared
        // and managed by the singleton pattern
        pyo3_asyncio::tokio::future_into_py(py, async move {
            Python::with_gil(|py| {
                Ok(py.None())
            })
        })
    }
}

/// Sync transport for httpx using Rust reqwest (blocking)
#[pyclass]
pub struct SyncTransport {
    client: reqwest::blocking::Client,
}

#[pymethods]
impl SyncTransport {
    #[new]
    fn new() -> PyResult<Self> {
        // Initialize tracing on first use
        crate::client::init_tracing();
        
        // Create a blocking client
        let client = reqwest::blocking::Client::builder()
            .pool_max_idle_per_host(64)
            .user_agent(format!("rust-httpx-transport/{}", env!("CARGO_PKG_VERSION")))
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(format!("Failed to create client: {}", e)))?;
        
        Ok(Self { client })
    }
    
    /// Handle a sync HTTP request
    fn handle_request(
        &self,
        py: Python,
        request: &PyAny,
    ) -> PyResult<PyObject> {
        // Extract request components
        let method = extract_method(request.getattr("method")?)?;
        let url = extract_url(request.getattr("url")?)?;
        let headers = extract_headers(request.getattr("headers")?)?;
        let extensions = extract_extensions(request.getattr("extensions")?)?;
        
        // Extract body - convert to bytes for sync client
        let body_bytes: Vec<u8> = if let Ok(py_body) = request.getattr("content") {
            // For sync transport, we need to extract the body as bytes
            if py_body.is_none() {
                Vec::new()
            } else if let Ok(py_bytes) = py_body.downcast::<pyo3::types::PyBytes>() {
                py_bytes.as_bytes().to_vec()
            } else if let Ok(py_str) = py_body.extract::<String>() {
                py_str.into_bytes()
            } else {
                return Err(pyo3::exceptions::PyTypeError::new_err(
                    "Sync transport only supports bytes or string bodies"
                ));
            }
        } else {
            Vec::new()
        };
        
        // Check configuration from extensions
        let timeout = extract_timeout_from_extensions(&extensions);
        let streaming = is_streaming_requested(&extensions);
        
        // Build request
        let mut req_builder = self.client.request(method, url)
            .headers(headers)
            .body(body_bytes);
        
        // Apply timeout if specified
        if let Some(timeout_duration) = timeout {
            req_builder = req_builder.timeout(timeout_duration);
        }
        
        // Execute the request (this will block)
        let response = req_builder.send()
            .map_err(TransportError::from)?;
        
        // Extract response components
        let status = response.status().as_u16();
        let response_headers = response.headers().clone();
        let response_extensions = Some(extensions.clone());
        
        if streaming {
            // Create streaming response
            let stream = SyncByteStream::from_response(response);
            let py_stream = Py::new(py, stream)?;
            
            create_response_object(
                py,
                status,
                response_headers,
                None,  // No content for streaming
                Some(py_stream.to_object(py)),
                response_extensions,
            )
        } else {
            // Read full response body
            let bytes = response.bytes()
                .map_err(TransportError::from)?;
            
            let py_content = PyBytes::new(py, &bytes);
            create_response_object(
                py,
                status,
                response_headers,
                Some(py_content.into()),
                None,  // No stream for non-streaming
                response_extensions,
            )
        }
    }
    
    /// Close the transport (cleanup)
    fn close(&self) -> PyResult<()> {
        // For now, we don't need to do anything
        Ok(())
    }
}

impl Default for AsyncTransport {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for SyncTransport {
    fn default() -> Self {
        Self::new().expect("Failed to create SyncTransport")
    }
} 