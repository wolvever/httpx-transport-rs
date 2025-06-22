#!/usr/bin/env python3
"""
Basic usage example for rust-httpx-transport.

This example demonstrates how to use the Rust-powered transport 
as a drop-in replacement for the standard httpx transport.
"""

import asyncio
import time
from typing import List

try:
    import httpx
    import rust_httpx
    
    DEPENDENCIES_AVAILABLE = True
except ImportError as e:
    print(f"Dependencies not available: {e}")
    print("Please install: pip install httpx rust-httpx-transport")
    DEPENDENCIES_AVAILABLE = False


async def async_example():
    """Example using the async Rust transport."""
    print("=== Async Transport Example ===")
    
    # Check if Rust transport is available
    if not rust_httpx.is_available():
        print("Rust transport not available, using standard httpx transport")
        transport = None
    else:
        print("Using Rust transport for improved performance!")
        transport = rust_httpx.AsyncTransport()
    
    # Create client with Rust transport
    async with httpx.AsyncClient(transport=transport) as client:
        print("Making a simple GET request...")
        response = await client.get("https://httpbin.org/get")
        print(f"Status: {response.status_code}")
        print(f"Headers: {dict(response.headers)}")
        
        print("\nMaking a POST request with JSON...")
        test_data = {"message": "Hello from Rust transport!", "timestamp": time.time()}
        response = await client.post("https://httpbin.org/post", json=test_data)
        print(f"Status: {response.status_code}")
        
        response_json = response.json()
        print(f"Echoed JSON: {response_json.get('json', {})}")
        
        print("\nTesting streaming...")
        async with client.stream("GET", "https://httpbin.org/stream/3") as stream_response:
            print(f"Stream status: {stream_response.status_code}")
            async for line in stream_response.aiter_lines():
                print(f"Stream line: {line[:50]}...")


def sync_example():
    """Example using the sync Rust transport."""
    print("\n=== Sync Transport Example ===")
    
    # Check if Rust transport is available
    if not rust_httpx.is_available():
        print("Rust transport not available, using standard httpx transport")
        transport = None
    else:
        print("Using Rust transport for improved performance!")
        transport = rust_httpx.SyncTransport()
    
    # Create client with Rust transport
    with httpx.Client(transport=transport) as client:
        print("Making a simple GET request...")
        response = client.get("https://httpbin.org/get")
        print(f"Status: {response.status_code}")
        
        print("Making a POST request with form data...")
        form_data = {"name": "Rust Transport", "performance": "excellent"}
        response = client.post("https://httpbin.org/post", data=form_data)
        print(f"Status: {response.status_code}")
        
        response_json = response.json()
        print(f"Echoed form: {response_json.get('form', {})}")


async def performance_comparison():
    """Simple performance comparison between transports."""
    print("\n=== Performance Comparison ===")
    
    urls = [
        "https://httpbin.org/get",
        "https://httpbin.org/json",
        "https://httpbin.org/headers",
        "https://httpbin.org/user-agent",
        "https://httpbin.org/gzip"
    ]
    
    # Test with standard transport
    print("Testing with standard httpx transport...")
    start_time = time.time()
    async with httpx.AsyncClient() as client:
        tasks = [client.get(url) for url in urls]
        responses = await asyncio.gather(*tasks)
    standard_time = time.time() - start_time
    print(f"Standard transport: {len(responses)} requests in {standard_time:.3f}s")
    
    # Test with Rust transport (if available)
    if rust_httpx.is_available():
        print("Testing with Rust transport...")
        start_time = time.time()
        async with httpx.AsyncClient(transport=rust_httpx.AsyncTransport()) as client:
            tasks = [client.get(url) for url in urls]
            responses = await asyncio.gather(*tasks)
        rust_time = time.time() - start_time
        print(f"Rust transport: {len(responses)} requests in {rust_time:.3f}s")
        
        if rust_time > 0:
            speedup = standard_time / rust_time
            print(f"Speedup: {speedup:.2f}x")
    else:
        print("Rust transport not available for comparison")


def version_info():
    """Display version and availability information."""
    print("\n=== Version Information ===")
    
    if DEPENDENCIES_AVAILABLE:
        info = rust_httpx.get_version_info()
        print(f"rust-httpx-transport version: {info['version']}")
        print(f"Rust extension available: {info['rust_available']}")
        if info['import_error']:
            print(f"Import error: {info['import_error']}")
        print(f"Python version: {'.'.join(map(str, info['python_version'][:3]))}")
        print(f"httpx version: {httpx.__version__}")
    else:
        print("Dependencies not available - cannot show version info")


async def main():
    """Run all examples."""
    print("Rust HTTP Transport Examples")
    print("=" * 40)
    
    if not DEPENDENCIES_AVAILABLE:
        return
    
    version_info()
    
    await async_example()
    sync_example()
    await performance_comparison()
    
    print("\n" + "=" * 40)
    print("Examples completed!")


if __name__ == "__main__":
    if DEPENDENCIES_AVAILABLE:
        asyncio.run(main())
    else:
        print("Cannot run examples - dependencies not available") 