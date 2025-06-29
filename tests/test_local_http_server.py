import http.server
from threading import Thread

import pytest

try:
    import httpcore
    HTTPCORE_AVAILABLE = True
except ImportError:
    HTTPCORE_AVAILABLE = False

try:
    import rust_httpx
    RUST_AVAILABLE = rust_httpx.is_available()
except ImportError:
    RUST_AVAILABLE = False

class HelloHandler(http.server.BaseHTTPRequestHandler):
    def do_GET(self):  # noqa: N802
        self.send_response(200)
        self.send_header("Content-Type", "text/plain")
        self.end_headers()
        self.wfile.write(b"hello from server")

    def log_message(self, *args, **kwargs):
        # Silence logging
        pass

@pytest.fixture
def http_server():
    server = http.server.HTTPServer(("127.0.0.1", 0), HelloHandler)
    port = server.server_address[1]
    thread = Thread(target=server.serve_forever, daemon=True)
    thread.start()

    yield f"http://127.0.0.1:{port}"

    server.shutdown()
    thread.join()

@pytest.mark.skipif(not RUST_AVAILABLE, reason="Rust transport not available")
@pytest.mark.skipif(not HTTPCORE_AVAILABLE, reason="httpcore not available")
@pytest.mark.asyncio
async def test_async_rust_transport_against_local_server(http_server):
    transport = rust_httpx.AsyncTransport()
    request = httpcore.Request("GET", http_server)
    response = await transport.handle_async_request(request)
    assert response.status == 200
    assert await response.aread() == b"hello from server"
    await transport.aclose()


@pytest.mark.skipif(not RUST_AVAILABLE, reason="Rust transport not available")
@pytest.mark.skipif(not HTTPCORE_AVAILABLE, reason="httpcore not available")
def test_sync_rust_transport_against_local_server(http_server):
    transport = rust_httpx.SyncTransport()
    request = httpcore.Request("GET", http_server)
    response = transport.handle_request(request)
    assert response.status == 200
    assert response.read() == b"hello from server"
    transport.close()
