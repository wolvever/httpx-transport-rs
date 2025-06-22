use std::sync::Arc;
use std::time::Duration;

use once_cell::sync::OnceCell;
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};

static CLIENT: OnceCell<Arc<ClientWithMiddleware>> = OnceCell::new();

/// Configuration for the HTTP client
#[derive(Debug, Clone)]
pub struct ClientConfig {
    pub timeout: Duration,
    pub pool_max_idle_per_host: usize,
    pub pool_idle_timeout: Duration,
    pub retries_max_attempts: u32,
    pub user_agent: String,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(30),
            pool_max_idle_per_host: 64,
            pool_idle_timeout: Duration::from_secs(90),
            retries_max_attempts: 3,
            user_agent: format!("rust-httpx-transport/{}", env!("CARGO_PKG_VERSION")),
        }
    }
}

/// Get or create the singleton HTTP client
pub fn get_client() -> Arc<ClientWithMiddleware> {
    CLIENT.get_or_init(|| {
        create_client(ClientConfig::default())
    }).clone()
}

/// Create a new HTTP client with middleware stack
fn create_client(config: ClientConfig) -> Arc<ClientWithMiddleware> {
    // Build the base reqwest client
    let base_client = reqwest::Client::builder()
        .timeout(config.timeout)
        .pool_max_idle_per_host(config.pool_max_idle_per_host)
        .pool_idle_timeout(config.pool_idle_timeout)
        .user_agent(config.user_agent)
        .http2_prior_knowledge()
        .use_rustls_tls()
        .build()
        .expect("Failed to create reqwest client");

    // For now, just use the basic client without complex middleware
    // TODO: Add proper middleware integration in future versions
    let client = ClientBuilder::new(base_client).build();

    Arc::new(client)
}

/// Initialize tracing subscriber for observability
pub fn init_tracing() {
    use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
    
    let _guard = tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer().with_target(false))
        .try_init();
} 