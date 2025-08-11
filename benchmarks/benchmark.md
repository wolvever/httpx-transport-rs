# HTTP Benchmark

Benchmark comparing Python httpx client with a Rust reqwest client against a simple Go server.

- **Requests**: 1000 sequential GET requests to `http://localhost:8000/`
- **Python (httpx)**: 0.9377s
- **Rust (reqwest)**: 0.1834s
- **Speedup**: 5.11x faster

Generated on: Mon Aug 11 12:35:26 UTC 2025
