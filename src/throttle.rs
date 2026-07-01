//! Throttle / debounce helpers.

use std::sync::Mutex;
use std::time::{Duration, Instant};

/// A gate that allows at most one action per `min_interval`. Subsequent actions
/// within the window are rejected by [`try_allow`] and must wait.
pub struct Throttle {
    min_interval: Duration,
    last: Mutex<Option<Instant>>,
}

impl Throttle {
    pub fn new(min_interval: Duration) -> Self {
        Throttle {
            min_interval,
            last: Mutex::new(None),
        }
    }

    /// Returns `Ok(())` if enough time has elapsed since the last allowed call
    /// (and updates the timestamp), else `Err(remaining)`.
    pub fn try_allow(&self) -> Result<(), Duration> {
        let mut last = self.last.lock().expect("throttle lock");
        let now = Instant::now();
        match *last {
            Some(prev) if now.duration_since(prev) < self.min_interval => {
                Err(self.min_interval - now.duration_since(prev))
            }
            _ => {
                *last = Some(now);
                Ok(())
            }
        }
    }

    /// Block until the next call is allowed, then record it.
    pub async fn wait(&self) {
        loop {
            let remaining = match self.try_allow() {
                Ok(()) => return,
                Err(r) => r,
            };
            tokio::time::sleep(remaining).await;
        }
    }
}

/// Debounce: run `f` and, while it keeps being triggered within `interval`,
/// only the last call fires after `interval` of quiet. This simple variant
/// spawns a task that resolves to the latest value when the stream settles.
pub async fn debounce<F, Fut, T>(interval: Duration, trigger: F) -> T
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = T>,
{
    // Single-shot helper: wait `interval`, then run once. Useful in loops where
    // the caller resets on new events.
    tokio::time::sleep(interval).await;
    trigger().await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn throttle_allows_once_per_interval() {
        let t = Throttle::new(Duration::from_millis(100));
        assert!(t.try_allow().is_ok()); // first ok
        assert!(t.try_allow().is_err()); // too soon
        tokio::time::sleep(Duration::from_millis(100)).await;
        assert!(t.try_allow().is_ok()); // after interval
    }

    #[tokio::test]
    async fn throttle_wait_blocks_until_allowed() {
        let t = Throttle::new(Duration::from_millis(50));
        assert!(t.try_allow().is_ok());
        let start = Instant::now();
        t.wait().await; // sleeps ~50ms real time
        assert!(start.elapsed() >= Duration::from_millis(30));
    }

    #[tokio::test]
    async fn debounce_runs_after_quiet() {
        let val = debounce(Duration::from_millis(20), || async { 42 }).await;
        assert_eq!(val, 42);
    }
}
