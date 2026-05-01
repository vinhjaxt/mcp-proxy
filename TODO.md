# Performance TODO — mcp-proxy

## Critical — ALL FIXED

### 1. ~~ProxyHandler: Mutex on every request kills concurrency~~ — FIXED
- **Fix**: `Arc<Mutex<RunningService>>` → `Arc<RunningService>`. rmcp methods take `&self`.
- **Tested**: 3 concurrent requests in 10ms.

### 2. ~~SseWriter::poll_write spawns a task per line~~ — FIXED
- **Fix**: Bounded `mpsc::channel(256)` + single sender task. `try_send` with `Full` logs warning and drops the message. This is **not** true backpressure (impossible in a sync `poll_write` with `mpsc`), but a bounded-drop strategy. With a 256-slot channel this should only trigger under extreme conditions. Empty strings are filtered in sender task to guard against edge cases.
- **Bug found during review**: Original fix sent empty strings into the channel during backpressure (which would be POSTed as invalid JSON), and lost the message that triggered backpressure. Fixed by using `try_send` + graceful drop with logging.

### 3. ~~SseWriter per-line clones~~ — FIXED
- **Fix**: Eliminated by channel approach.

### 4. ~~SseWriter silently drops errors~~ — FIXED
- **Fix**: Returns `Err(InvalidData)` for bad UTF-8 or invalid JSON. `Err(BrokenPipe)` for closed channel. Channel-full is logged as a warning.

### 7. ~~SseWriter JSON round-trip~~ — FIXED
- **Fix**: Uses `serde::de::IgnoredAny` for zero-allocation JSON validation instead of parsing into `serde_json::Value` (which allocates a full JSON tree). Body is forwarded as-is without re-serialization.

## High

### 5. Streamable HTTP server: block_in_place per session — WONTFIX (upstream API)
- **Verified**: rmcp `StreamableHttpService::new` requires `Fn() -> Result<S>` — synchronous factory. `block_in_place` is the correct tokio pattern for bridging sync→async. Cannot fix without upstream API change to accept `async fn` factories.

### 6. ~~reqwest::Client built without connection pool tuning~~ — FIXED
- **Fix**: Added `connect_timeout(10s)`, `pool_idle_timeout(90s)`, `tcp_keepalive(30s)` to all `reqwest::Client` builders across `auth.rs`, `sse_client.rs`, `streamable_http_client.rs`.

### 8. ~~SSE client creates two reqwest::Client instances~~ — FIXED
- **Fix**: Single client, auth header applied per-request instead of via default headers.

## Medium

### 9. ~~OidcConfigCache uses tokio::sync::RwLock~~ — FIXED
- **Fix**: Changed `tokio::sync::RwLock` → `std::sync::RwLock` for `oidc_config_cache`, `client_registration`, and `auth_state` in `auth.rs`. These hold small data with fast reads, no need for async lock overhead.

### 10. ~~No BufReader on child process stdio~~ — FIXED
- **Fix**: Added `tokio::io::BufReader::with_capacity(8192, read)` to both `sse_server.rs` and `streamable_http_server.rs` via `tokio_process.split()` + `(read, write)` tuple transport.

### 11. ConfigManager uses blocking std::fs — DEFERRED
- **Reason**: File I/O only occurs on the auth/config path (startup, 401 token refresh), not on the proxy hot path. Files are tiny (< 1KB). Practical impact is negligible. Fix would require converting the entire `ConfigManager` API to async, adding complexity for no measurable gain in a proxy use case.

### 12. ~~Coordination spawns OAuth server without graceful shutdown~~ — FIXED
- **Fix**: Added `CancellationToken` + `with_graceful_shutdown` to OAuth callback server. Replaced `abort()` + `sleep(1s)` with `cancel()` + `await`.

## Low

### 13. ~~clone() on request objects in ProxyHandler~~ — FIXED (by #1 Mutex removal)
- `call_tool` no longer clones the request. All proxy methods call `self.client.method()` directly.

### 14. ~~Duplicate is_lock_expired functions~~ — FIXED
- **Fix**: Moved to `utils.rs` as `pub fn is_lock_expired`. Both `config.rs` and `coordination.rs` now delegate to it.

### 15. ~~Empty error swallowed in SseWriter~~ — FIXED (by #4 fix)

### 16. ~~process::Command for `ps` to check running processes~~ — FIXED
- **Fix**: Replaced `Command::new("ps")` with `unsafe { libc::kill(pid as i32, 0) == 0 }`. Added `libc = "0.2"` as direct dependency. Zero-cost syscall vs fork+exec.

### 17. ~~Hardcoded magic numbers~~ — FIXED
- **Fix**: Centralized into `utils.rs` as constants: `HTTP_TIMEOUT`, `HTTP_CONNECT_TIMEOUT`, `HTTP_POOL_IDLE_TIMEOUT`, `HTTP_TCP_KEEPALIVE`, `OIDC_CACHE_TTL`, `LOCK_EXPIRY_SECS`, `SSE_KEEP_ALIVE`. All call sites updated.

### 18. Release profile optimization — DEFERRED
- **Reason**: `opt-level = "z"` with LTO produces a 4.7 MB binary. Switching to `opt-level = 3` would increase size for marginal performance gains on a network-bound proxy. Trade-off favors small binary for container deployment.
