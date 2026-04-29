//! In-process sliding-window rate limiter.
//!
//! Keys are arbitrary strings — callers compose them as `<scope>:<key>`
//! (e.g. `"login:ip:1.2.3.4"`). On limit exceeded, callers receive the
//! number of seconds until the oldest entry exits the window.
//!
//! Storage is a `HashMap<String, Vec<Instant>>` behind a `Mutex`. Entries
//! older than the window are pruned lazily on each `check`. This is fine
//! for M1 traffic; if cardinality grows we can swap in a per-key TTL map
//! or move to Redis without changing call sites.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

#[derive(Clone, Default)]
pub struct RateLimiter {
    inner: Arc<Mutex<HashMap<String, Vec<Instant>>>>,
}

impl RateLimiter {
    pub fn new() -> Self {
        Self::default()
    }

    /// Allow up to `limit` events per `window` for `key`.
    /// Returns `Ok(())` and records the hit, or `Err(retry_after_seconds)`
    /// if denied.
    pub fn check(&self, key: &str, limit: usize, window: Duration) -> Result<(), u64> {
        let mut map = self.inner.lock().expect("rate limiter mutex poisoned");
        let now = Instant::now();
        let cutoff = now.checked_sub(window).unwrap_or(now);

        let entries = map.entry(key.to_string()).or_default();
        entries.retain(|t| *t >= cutoff);

        if entries.len() >= limit {
            let oldest = entries[0];
            let retry = (oldest + window).saturating_duration_since(now);
            return Err(retry.as_secs().max(1));
        }

        entries.push(now);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allows_under_limit() {
        let rl = RateLimiter::new();
        for _ in 0..3 {
            assert!(rl.check("k", 3, Duration::from_secs(60)).is_ok());
        }
    }

    #[test]
    fn rejects_at_limit() {
        let rl = RateLimiter::new();
        assert!(rl.check("k", 2, Duration::from_secs(60)).is_ok());
        assert!(rl.check("k", 2, Duration::from_secs(60)).is_ok());
        let err = rl.check("k", 2, Duration::from_secs(60)).unwrap_err();
        assert!(err >= 1 && err <= 60);
    }

    #[test]
    fn keys_are_independent() {
        let rl = RateLimiter::new();
        assert!(rl.check("a", 1, Duration::from_secs(60)).is_ok());
        assert!(rl.check("a", 1, Duration::from_secs(60)).is_err());
        assert!(rl.check("b", 1, Duration::from_secs(60)).is_ok());
    }

    #[test]
    fn window_expiry_clears_entries() {
        let rl = RateLimiter::new();
        assert!(rl.check("k", 1, Duration::from_millis(50)).is_ok());
        assert!(rl.check("k", 1, Duration::from_millis(50)).is_err());
        std::thread::sleep(Duration::from_millis(60));
        assert!(rl.check("k", 1, Duration::from_millis(50)).is_ok());
    }
}
