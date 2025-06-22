"""
High-performance Rust transport for Python httpx.

This module provides drop-in replacements for httpx transports using Rust's
reqwest library with tokio and hyper for improved performance.
"""

import sys
from typing import Any, Optional, TYPE_CHECKING

if TYPE_CHECKING:
    import httpx
    import httpcore

try:
    from ._rust_httpx import AsyncTransport as _AsyncTransport, SyncTransport as _SyncTransport
    from ._rust_httpx import __version__
    
    _RUST_AVAILABLE = True
except ImportError as e:
    _RUST_AVAILABLE = False
    _IMPORT_ERROR = e
    
    # Fallback version
    __version__ = "0.1.0"


class AsyncTransport:
    """
    High-performance async transport for httpx using Rust.
    
    This transport implements the httpx.AsyncBaseTransport interface
    and provides a drop-in replacement for the default httpx transport.
    
    Example:
        import httpx
        import rust_httpx
        
        async with httpx.AsyncClient(transport=rust_httpx.AsyncTransport()) as client:
            response = await client.get("https://api.example.com/data")
    """
    
    def __init__(self, **kwargs: Any) -> None:
        if not _RUST_AVAILABLE:
            raise ImportError(
                f"Rust extension not available. Please ensure the rust-httpx-transport "
                f"package is properly installed. Original error: {_IMPORT_ERROR}"
            )
        
        self._transport = _AsyncTransport()
    
    async def handle_async_request(self, request: "httpcore.Request") -> "httpcore.Response":
        """Handle an async HTTP request."""
        return await self._transport.handle_async_request(request)
    
    async def aclose(self) -> None:
        """Close the transport and clean up resources."""
        await self._transport.aclose()
    
    def __repr__(self) -> str:
        return f"{self.__class__.__name__}()"


class SyncTransport:
    """
    High-performance sync transport for httpx using Rust.
    
    This transport implements the httpx.BaseTransport interface
    and provides a drop-in replacement for the default httpx transport.
    
    Example:
        import httpx
        import rust_httpx
        
        with httpx.Client(transport=rust_httpx.SyncTransport()) as client:
            response = client.get("https://api.example.com/data")
    """
    
    def __init__(self, **kwargs: Any) -> None:
        if not _RUST_AVAILABLE:
            raise ImportError(
                f"Rust extension not available. Please ensure the rust-httpx-transport "
                f"package is properly installed. Original error: {_IMPORT_ERROR}"
            )
        
        self._transport = _SyncTransport()
    
    def handle_request(self, request: "httpcore.Request") -> "httpcore.Response":
        """Handle a sync HTTP request."""
        return self._transport.handle_request(request)
    
    def close(self) -> None:
        """Close the transport and clean up resources."""
        self._transport.close()
    
    def __repr__(self) -> str:
        return f"{self.__class__.__name__}()"


def is_available() -> bool:
    """Check if the Rust transport is available."""
    return _RUST_AVAILABLE


def get_version_info() -> dict[str, Any]:
    """Get version and availability information."""
    return {
        "version": __version__,
        "rust_available": _RUST_AVAILABLE,
        "import_error": str(_IMPORT_ERROR) if not _RUST_AVAILABLE else None,
        "python_version": sys.version_info,
    }


# Export the main classes and functions
__all__ = [
    "AsyncTransport",
    "SyncTransport", 
    "is_available",
    "get_version_info",
    "__version__",
] 