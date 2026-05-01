use crate::config::LockFile;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub const DEFAULT_CALLBACK_PORT: u16 = 9292;
pub const DEFAULT_COORDINATION_TIMEOUT: u64 = 30;

// HTTP client timeouts
pub const HTTP_TIMEOUT: Duration = Duration::from_secs(30);
pub const HTTP_CONNECT_TIMEOUT: Duration = Duration::from_secs(10);
pub const HTTP_POOL_IDLE_TIMEOUT: Duration = Duration::from_secs(90);
pub const HTTP_TCP_KEEPALIVE: Duration = Duration::from_secs(30);

// Auth/config timeouts
pub const OIDC_CACHE_TTL: Duration = Duration::from_secs(3600);
pub const LOCK_EXPIRY_SECS: u64 = 600;

// Server keep-alive
pub const SSE_KEEP_ALIVE: Duration = Duration::from_secs(15);

pub fn hash_server_url(server_url: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    server_url.hash(&mut hasher);
    format!("{:x}", hasher.finish())
}

pub fn is_lock_expired(lock: &LockFile) -> bool {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    now > lock.created_at + LOCK_EXPIRY_SECS
}
