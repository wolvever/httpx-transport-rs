use std::sync::Arc;

use bytes::Bytes;
use futures::StreamExt;
use pyo3::prelude::*;
use pyo3::types::PyBytes;
use tokio::sync::mpsc;

use crate::errors::TransportError;

/// A streaming response body that can be consumed from Python
#[pyclass]
pub struct ByteStream {
    receiver: Arc<tokio::sync::Mutex<mpsc::Receiver<Result<Bytes, TransportError>>>>,
}

impl ByteStream {
    /// Create a new ByteStream from a reqwest response body
    pub fn from_response(response: reqwest::Response) -> Self {
        let (tx, rx) = mpsc::channel(32);
        let mut stream = response.bytes_stream();
        
        // Spawn a task to forward the stream to the channel
        tokio::spawn(async move {
            while let Some(result) = stream.next().await {
                let bytes_result = result.map_err(TransportError::from);
                if tx.send(bytes_result).await.is_err() {
                    break; // Receiver dropped
                }
            }
        });
        
        Self {
            receiver: Arc::new(tokio::sync::Mutex::new(rx)),
        }
    }
    
    /// Create a new ByteStream from a bytes iterator
    pub fn from_bytes_iter<I>(iter: I) -> Self 
    where
        I: Iterator<Item = Result<Bytes, TransportError>> + Send + 'static,
    {
        let (tx, rx) = mpsc::channel(32);
        
        // Spawn a task to forward the iterator to the channel
        tokio::spawn(async move {
            for result in iter {
                if tx.send(result).await.is_err() {
                    break; // Receiver dropped
                }
            }
        });
        
        Self {
            receiver: Arc::new(tokio::sync::Mutex::new(rx)),
        }
    }
}

#[pymethods]
impl ByteStream {
    /// Get the next chunk of bytes (async) - simplified version
    fn read_chunk<'py>(&self, py: Python<'py>) -> PyResult<&'py PyAny> {
        let receiver = self.receiver.clone();
        
        pyo3_asyncio::tokio::future_into_py(py, async move {
            let mut rx = receiver.lock().await;
            match rx.recv().await {
                Some(Ok(bytes)) => {
                    Python::with_gil(|py| {
                        let py_bytes = PyBytes::new(py, &bytes);
                        Ok::<PyObject, PyErr>(py_bytes.into())
                    })
                }
                Some(Err(e)) => Err(PyErr::from(e)),
                None => {
                    Python::with_gil(|py| {
                        Ok::<PyObject, PyErr>(py.None())
                    })
                }
            }
        })
    }
}

/// A synchronous version of ByteStream for blocking operations
#[pyclass]
pub struct SyncByteStream {
    bytes_vec: Vec<Result<Bytes, TransportError>>,
    index: usize,
}

impl SyncByteStream {
    /// Create a new SyncByteStream from a response body
    pub fn from_response(response: reqwest::blocking::Response) -> Self {
        let mut bytes_vec = Vec::new();
        
        // For blocking responses, we'll just read the full body at once
        // This is simpler and more appropriate for the sync API
        match response.bytes() {
            Ok(bytes) => {
                bytes_vec.push(Ok(bytes));
            }
            Err(e) => {
                bytes_vec.push(Err(TransportError::from(e)));
            }
        }
        
        Self {
            bytes_vec,
            index: 0,
        }
    }
}

#[pymethods]
impl SyncByteStream {
    /// Get the next chunk of bytes (sync) - simplified version
    fn read_chunk(&mut self, py: Python) -> PyResult<PyObject> {
        if self.index >= self.bytes_vec.len() {
            return Ok(py.None());
        }
        
        let result = &self.bytes_vec[self.index];
        self.index += 1;
        
        match result {
            Ok(bytes) => {
                let py_bytes = PyBytes::new(py, bytes);
                Ok(py_bytes.into())
            }
            Err(e) => Err(PyErr::from(e.clone())),
        }
    }
}

/// Utility functions for handling Python request bodies
pub fn extract_body_from_python(py_body: &PyAny) -> PyResult<reqwest::Body> {
    if py_body.is_none() {
        return Ok(reqwest::Body::from(""));
    }
    
    // Try to extract as bytes first
    if let Ok(py_bytes) = py_body.downcast::<PyBytes>() {
        let bytes = py_bytes.as_bytes();
        return Ok(reqwest::Body::from(bytes.to_vec()));
    }
    
    // Try to extract as string
    if let Ok(py_str) = py_body.extract::<String>() {
        return Ok(reqwest::Body::from(py_str));
    }
    
    // Try to extract as iterator
    if let Ok(py_iter) = py_body.iter() {
        let mut body_data = Vec::new();
        for item in py_iter {
            let item = item?;
            if let Ok(chunk_bytes) = item.downcast::<PyBytes>() {
                body_data.extend_from_slice(chunk_bytes.as_bytes());
            } else if let Ok(chunk_str) = item.extract::<String>() {
                body_data.extend_from_slice(chunk_str.as_bytes());
            } else {
                return Err(pyo3::exceptions::PyTypeError::new_err(
                    "Body iterator must yield bytes or strings"
                ));
            }
        }
        return Ok(reqwest::Body::from(body_data));
    }
    
    Err(pyo3::exceptions::PyTypeError::new_err(
        "Body must be bytes, string, or iterator"
    ))
} 