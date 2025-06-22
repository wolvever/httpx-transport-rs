"""Tests for the Rust httpx transport."""

import pytest
import asyncio
from unittest.mock import Mock, patch

try:
    import httpx
    import httpcore
    HTTPX_AVAILABLE = True
except ImportError:
    HTTPX_AVAILABLE = False

try:
    import rust_httpx
    RUST_AVAILABLE = rust_httpx.is_available()
except ImportError:
    RUST_AVAILABLE = False


@pytest.mark.skipif(not RUST_AVAILABLE, reason="Rust transport not available")
@pytest.mark.skipif(not HTTPX_AVAILABLE, reason="httpx not available")
class TestAsyncTransport:
    """Test the async Rust transport."""
    
    def test_transport_creation(self):
        """Test that we can create an AsyncTransport."""
        transport = rust_httpx.AsyncTransport()
        assert isinstance(transport, rust_httpx.AsyncTransport)
    
    @pytest.mark.asyncio
    async def test_simple_get_request(self):
        """Test a simple GET request."""
        transport = rust_httpx.AsyncTransport()
        
        async with httpx.AsyncClient(transport=transport) as client:
            # Use httpbin.org for testing
            response = await client.get("https://httpbin.org/get")
            assert response.status_code == 200
            assert "httpbin.org" in response.text
    
    @pytest.mark.asyncio
    async def test_post_request_with_json(self):
        """Test a POST request with JSON data."""
        transport = rust_httpx.AsyncTransport()
        
        async with httpx.AsyncClient(transport=transport) as client:
            test_data = {"key": "value", "number": 42}
            response = await client.post("https://httpbin.org/post", json=test_data)
            assert response.status_code == 200
            
            response_data = response.json()
            assert response_data["json"] == test_data
    
    @pytest.mark.asyncio
    async def test_custom_headers(self):
        """Test request with custom headers."""
        transport = rust_httpx.AsyncTransport()
        
        async with httpx.AsyncClient(transport=transport) as client:
            headers = {"Custom-Header": "test-value", "User-Agent": "rust-httpx-test"}
            response = await client.get("https://httpbin.org/headers", headers=headers)
            assert response.status_code == 200
            
            response_data = response.json()
            assert "Custom-Header" in response_data["headers"]
            assert response_data["headers"]["Custom-Header"] == "test-value"
    
    @pytest.mark.asyncio
    async def test_timeout_configuration(self):
        """Test timeout configuration."""
        transport = rust_httpx.AsyncTransport()
        
        async with httpx.AsyncClient(transport=transport, timeout=5.0) as client:
            # This should work within the timeout
            response = await client.get("https://httpbin.org/get")
            assert response.status_code == 200
    
    @pytest.mark.asyncio
    async def test_streaming_response(self):
        """Test streaming response."""
        transport = rust_httpx.AsyncTransport()
        
        async with httpx.AsyncClient(transport=transport) as client:
            async with client.stream("GET", "https://httpbin.org/stream/3") as response:
                assert response.status_code == 200
                
                lines = []
                async for line in response.aiter_lines():
                    lines.append(line)
                
                assert len(lines) == 3  # httpbin.org/stream/3 returns 3 lines


@pytest.mark.skipif(not RUST_AVAILABLE, reason="Rust transport not available")
@pytest.mark.skipif(not HTTPX_AVAILABLE, reason="httpx not available")
class TestSyncTransport:
    """Test the sync Rust transport."""
    
    def test_transport_creation(self):
        """Test that we can create a SyncTransport."""
        transport = rust_httpx.SyncTransport()
        assert isinstance(transport, rust_httpx.SyncTransport)
    
    def test_simple_get_request(self):
        """Test a simple GET request."""
        transport = rust_httpx.SyncTransport()
        
        with httpx.Client(transport=transport) as client:
            response = client.get("https://httpbin.org/get")
            assert response.status_code == 200
            assert "httpbin.org" in response.text
    
    def test_post_request_with_json(self):
        """Test a POST request with JSON data."""
        transport = rust_httpx.SyncTransport()
        
        with httpx.Client(transport=transport) as client:
            test_data = {"key": "value", "number": 42}
            response = client.post("https://httpbin.org/post", json=test_data)
            assert response.status_code == 200
            
            response_data = response.json()
            assert response_data["json"] == test_data
    
    def test_custom_headers(self):
        """Test request with custom headers."""
        transport = rust_httpx.SyncTransport()
        
        with httpx.Client(transport=transport) as client:
            headers = {"Custom-Header": "test-value", "User-Agent": "rust-httpx-test"}
            response = client.get("https://httpbin.org/headers", headers=headers)
            assert response.status_code == 200
            
            response_data = response.json()
            assert "Custom-Header" in response_data["headers"]
            assert response_data["headers"]["Custom-Header"] == "test-value"


@pytest.mark.skipif(not RUST_AVAILABLE, reason="Rust transport not available")
class TestUtilities:
    """Test utility functions."""
    
    def test_is_available(self):
        """Test the is_available function."""
        assert rust_httpx.is_available() is True
    
    def test_get_version_info(self):
        """Test the get_version_info function."""
        info = rust_httpx.get_version_info()
        
        assert "version" in info
        assert "rust_available" in info
        assert "python_version" in info
        assert info["rust_available"] is True
        assert info["import_error"] is None


class TestFallbackBehavior:
    """Test behavior when Rust extension is not available."""
    
    @patch('rust_httpx._RUST_AVAILABLE', False)
    @patch('rust_httpx._IMPORT_ERROR', ImportError("Mock import error"))
    def test_transport_creation_fails_gracefully(self):
        """Test that transport creation fails gracefully when Rust is not available."""
        with pytest.raises(ImportError, match="Rust extension not available"):
            rust_httpx.AsyncTransport()
        
        with pytest.raises(ImportError, match="Rust extension not available"):
            rust_httpx.SyncTransport() 