import subprocess
import time
from pathlib import Path

import httpx

SERVER_CMD = ["go", "run", "benchmarks/server.go"]
RUST_CLIENT_MANIFEST = Path("benchmarks/rust-client/Cargo.toml")
RUST_BINARY = Path("benchmarks/rust-client/target/release/rust-client")
URL = "http://localhost:8000/"


def start_server():
    return subprocess.Popen(
        SERVER_CMD,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
    )


def wait_for_server(timeout: float = 5.0) -> None:
    start = time.time()
    while time.time() - start < timeout:
        try:
            httpx.get(URL)
            return
        except httpx.HTTPError:
            time.sleep(0.1)
    raise RuntimeError("Server did not start in time")


def benchmark_python(requests: int) -> float:
    with httpx.Client() as client:
        start = time.perf_counter()
        for _ in range(requests):
            r = client.get(URL)
            r.raise_for_status()
        end = time.perf_counter()
    return end - start


def benchmark_rust(requests: int) -> float:
    if not RUST_BINARY.exists():
        subprocess.check_call(
            ["cargo", "build", "--release", "--manifest-path", str(RUST_CLIENT_MANIFEST)],
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
        )
    result = subprocess.check_output([str(RUST_BINARY), str(requests)])
    return float(result.strip())


def main():
    requests = 1000
    server = start_server()
    try:
        wait_for_server()
        py_time = benchmark_python(requests)
        rust_time = benchmark_rust(requests)
        speedup = py_time / rust_time
        print(f"Python transport: {py_time:.4f}s")
        print(f"Rust transport: {rust_time:.4f}s")
        print(f"Speedup: {speedup:.2f}x")
    finally:
        server.terminate()
        server.wait()


if __name__ == "__main__":
    main()
