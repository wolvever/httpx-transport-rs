use pyo3::prelude::*;

mod transport;
mod client;
mod streaming;
mod errors;
mod utils;

use transport::{AsyncTransport, SyncTransport};

/// High-performance Rust transport for Python httpx
#[pymodule]
fn _rust_httpx(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<AsyncTransport>()?;
    m.add_class::<SyncTransport>()?;
    
    // Add version info
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    
    Ok(())
} 