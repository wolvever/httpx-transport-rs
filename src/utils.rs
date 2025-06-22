use std::collections::HashMap;
use std::str::FromStr;

use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList, PyTuple};
use reqwest::{Method, Url};



/// Extract HTTP method from Python request
pub fn extract_method(py_method: &PyAny) -> PyResult<Method> {
    let method_str: String = py_method.extract()?;
    Method::from_str(&method_str)
        .map_err(|_| pyo3::exceptions::PyValueError::new_err(format!("Invalid HTTP method: {}", method_str)))
}

/// Extract URL from Python request
pub fn extract_url(py_url: &PyAny) -> PyResult<Url> {
    let url_str: String = py_url.extract()?;
    Url::parse(&url_str)
        .map_err(|e| pyo3::exceptions::PyValueError::new_err(format!("Invalid URL: {}", e)))
}

/// Extract headers from Python request
pub fn extract_headers(py_headers: &PyAny) -> PyResult<reqwest::header::HeaderMap> {
    let mut headers = reqwest::header::HeaderMap::new();
    
    if py_headers.is_none() {
        return Ok(headers);
    }
    
    // Handle different header formats
    if let Ok(py_dict) = py_headers.downcast::<PyDict>() {
        for (key, value) in py_dict {
            let key_str: String = key.extract()?;
            let value_str: String = value.extract()?;
            
            let header_name = reqwest::header::HeaderName::from_str(&key_str)
                .map_err(|e| pyo3::exceptions::PyValueError::new_err(format!("Invalid header name: {}", e)))?;
            let header_value = reqwest::header::HeaderValue::from_str(&value_str)
                .map_err(|e| pyo3::exceptions::PyValueError::new_err(format!("Invalid header value: {}", e)))?;
            
            headers.insert(header_name, header_value);
        }
    } else if let Ok(py_list) = py_headers.downcast::<PyList>() {
        // Handle list of tuples format: [("name", "value"), ...]
        for item in py_list {
            let tuple: &PyTuple = item.downcast()?;
            if tuple.len() != 2 {
                return Err(pyo3::exceptions::PyValueError::new_err("Header tuples must have exactly 2 elements"));
            }
            
            let key_str: String = tuple.get_item(0)?.extract()?;
            let value_str: String = tuple.get_item(1)?.extract()?;
            
            let header_name = reqwest::header::HeaderName::from_str(&key_str)
                .map_err(|e| pyo3::exceptions::PyValueError::new_err(format!("Invalid header name: {}", e)))?;
            let header_value = reqwest::header::HeaderValue::from_str(&value_str)
                .map_err(|e| pyo3::exceptions::PyValueError::new_err(format!("Invalid header value: {}", e)))?;
            
            headers.insert(header_name, header_value);
        }
    }
    
    Ok(headers)
}

/// Convert Rust response headers to Python format
pub fn convert_headers_to_python(headers: &reqwest::header::HeaderMap, py: Python) -> PyResult<PyObject> {
    let py_list = PyList::empty(py);
    
    for (name, value) in headers {
        let name_str = name.as_str();
        let value_str = value.to_str()
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(format!("Invalid header value: {}", e)))?;
        
        let tuple = PyTuple::new(py, &[name_str, value_str]);
        py_list.append(tuple)?;
    }
    
    Ok(py_list.into())
}

/// Extract extensions from Python request
pub fn extract_extensions(py_extensions: &PyAny) -> PyResult<HashMap<String, serde_json::Value>> {
    let mut extensions = HashMap::new();
    
    if py_extensions.is_none() {
        return Ok(extensions);
    }
    
    if let Ok(py_dict) = py_extensions.downcast::<PyDict>() {
        for (key, value) in py_dict {
            let key_str: String = key.extract()?;
            
            // Convert Python value to JSON value for processing
            let json_value = if value.is_none() {
                serde_json::Value::Null
            } else if let Ok(b) = value.extract::<bool>() {
                serde_json::Value::Bool(b)
            } else if let Ok(i) = value.extract::<i64>() {
                serde_json::Value::Number(serde_json::Number::from(i))
            } else if let Ok(f) = value.extract::<f64>() {
                if let Some(num) = serde_json::Number::from_f64(f) {
                    serde_json::Value::Number(num)
                } else {
                    serde_json::Value::Null
                }
            } else if let Ok(s) = value.extract::<String>() {
                serde_json::Value::String(s)
            } else {
                // Try to convert to string as fallback
                let s: String = value.str()?.extract()?;
                serde_json::Value::String(s)
            };
            
            extensions.insert(key_str, json_value);
        }
    }
    
    Ok(extensions)
}

/// Create Python response object from Rust response
pub fn create_response_object(
    py: Python,
    status: u16,
    headers: reqwest::header::HeaderMap,
    content: Option<PyObject>,
    stream: Option<PyObject>,
    extensions: Option<HashMap<String, serde_json::Value>>,
) -> PyResult<PyObject> {
    // Import httpcore Response class
    let httpcore = py.import("httpcore")?;
    let response_class = httpcore.getattr("Response")?;
    
    // Convert headers
    let py_headers = convert_headers_to_python(&headers, py)?;
    
    // Create response kwargs
    let kwargs = PyDict::new(py);
    kwargs.set_item("status", status)?;
    kwargs.set_item("headers", py_headers)?;
    
    if let Some(content) = content {
        kwargs.set_item("content", content)?;
    }
    
    if let Some(stream) = stream {
        kwargs.set_item("stream", stream)?;
    }
    
    if let Some(ext) = extensions {
        let py_extensions = PyDict::new(py);
        for (key, value) in ext {
            let py_value = match value {
                serde_json::Value::Null => py.None(),
                serde_json::Value::Bool(b) => b.into_py(py),
                serde_json::Value::Number(n) => {
                    if let Some(i) = n.as_i64() {
                        i.into_py(py)
                    } else if let Some(f) = n.as_f64() {
                        f.into_py(py)
                    } else {
                        py.None()
                    }
                }
                serde_json::Value::String(s) => s.into_py(py),
                _ => py.None(),
            };
            py_extensions.set_item(key, py_value)?;
        }
        kwargs.set_item("extensions", py_extensions)?;
    }
    
    // Create and return response object
    let response = response_class.call((), Some(kwargs))?;
    Ok(response.to_object(py))
}

/// Extract timeout configuration from extensions
pub fn extract_timeout_from_extensions(extensions: &HashMap<String, serde_json::Value>) -> Option<std::time::Duration> {
    if let Some(timeout_value) = extensions.get("timeout") {
        match timeout_value {
            serde_json::Value::Number(n) => {
                if let Some(seconds) = n.as_f64() {
                    if seconds > 0.0 {
                        return Some(std::time::Duration::from_secs_f64(seconds));
                    }
                }
            }
            _ => {}
        }
    }
    None
}

/// Check if streaming is requested in extensions
pub fn is_streaming_requested(extensions: &HashMap<String, serde_json::Value>) -> bool {
    extensions.get("stream")
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
} 