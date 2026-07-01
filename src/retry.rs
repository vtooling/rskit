//! Retry with exponential backoff.

use std::time::Duration;

use anyhow::Result;

/// Configurable retry policy with exponential backoff.
pub struct Retry {
    max_attempts: usize,
    initial_delay: Duration,
    factor: f64,
    max_delay: Duration,
}

impl Retry {
    pub fn new() -> Self {
        Retry {
            max_attempts: 3,
            initial_delay: Duration::from_millis(100),
            factor: 2.0,
            max_delay: Duration::from_secs(30),
        }
    }

    /// Total attempts (including the first). Default 3.
    pub fn max_attempts(mut self, n: usize) -> Self {
        self.max_attempts = n.max(1);
        self
    }

    /// Delay before the first retry. Default 100ms.
    pub fn initial_delay(mut self, d: Duration) -> Self {
        self.initial_delay = d;
        self
    }

    /// Backoff multiplier applied after each failure. Default 2.0.
    pub fn factor(mut self, f: f64) -> Self {
        self.factor = f.max(1.0);
        self
    }

    /// Cap on the delay between attempts. Default 30s.
    pub fn max_delay(mut self, d: Duration) -> Self {
        self.max_delay = d;
        self
    }

    fn next_delay(&self, current: Duration) -> Duration {
        let ms = (current.as_millis() as f64 * self.factor) as u64;
        Duration::from_millis(ms).min(self.max_delay)
    }

    /// Run an async closure, retrying on `Err`. Returns the last error if all
    /// attempts fail.
    pub async fn run_async<F, Fut, T>(&self, mut f: F) -> Result<T>
    where
        F: FnMut() -> Fut,
        Fut: std::future::Future<Output = Result<T>>,
    {
        let mut delay = self.initial_delay;
        let mut last_err: Option<anyhow::Error> = None;
        for attempt in 1..=self.max_attempts {
            match f().await {
                Ok(v) => return Ok(v),
                Err(e) => {
                    last_err = Some(e);
                    if attempt == self.max_attempts {
                        break;
                    }
                    tokio::time::sleep(delay).await;
                    delay = self.next_delay(delay);
                }
            }
        }
        Err(last_err.unwrap_or_else(|| anyhow::anyhow!("retry failed with no error captured")))
    }

    /// Synchronous variant. Uses `std::thread::sleep`.
    pub fn run_sync<F, T>(&self, mut f: F) -> Result<T>
    where
        F: FnMut() -> Result<T>,
    {
        let mut delay = self.initial_delay;
        let mut last_err: Option<anyhow::Error> = None;
        for attempt in 1..=self.max_attempts {
            match f() {
                Ok(v) => return Ok(v),
                Err(e) => {
                    last_err = Some(e);
                    if attempt == self.max_attempts {
                        break;
                    }
                    std::thread::sleep(delay);
                    delay = self.next_delay(delay);
                }
            }
        }
        Err(last_err.unwrap_or_else(|| anyhow::anyhow!("retry failed with no error captured")))
    }
}

impl Default for Retry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{
        Arc,
        atomic::{AtomicU32, Ordering},
    };

    #[tokio::test(start_paused = true)]
    async fn async_succeeds_after_retries() {
        let attempts = Arc::new(AtomicU32::new(0));
        let a = attempts.clone();
        let res = Retry::new()
            .max_attempts(3)
            .initial_delay(Duration::from_millis(10))
            .run_async(move || {
                let a = a.clone();
                async move {
                    let n = a.fetch_add(1, Ordering::SeqCst) + 1;
                    if n < 3 {
                        anyhow::bail!("fail #{n}")
                    }
                    Ok("done")
                }
            })
            .await;
        assert_eq!(res.unwrap(), "done");
        assert_eq!(attempts.load(Ordering::SeqCst), 3);
    }

    #[tokio::test(start_paused = true)]
    async fn async_exhausts_attempts() {
        let attempts = Arc::new(AtomicU32::new(0));
        let a = attempts.clone();
        let res: anyhow::Result<&str> = Retry::new()
            .max_attempts(2)
            .initial_delay(Duration::from_millis(5))
            .run_async(move || {
                let a = a.clone();
                async move {
                    a.fetch_add(1, Ordering::SeqCst);
                    anyhow::bail!("always")
                }
            })
            .await;
        assert!(res.is_err());
        assert_eq!(attempts.load(Ordering::SeqCst), 2);
    }

    #[test]
    fn sync_succeeds_eventually() {
        let attempts = Arc::new(AtomicU32::new(0));
        let a = attempts.clone();
        let res = Retry::new()
            .max_attempts(3)
            .initial_delay(Duration::from_millis(1))
            .run_sync(move || {
                let n = a.fetch_add(1, Ordering::SeqCst) + 1;
                if n < 2 {
                    anyhow::bail!("fail")
                }
                Ok(42)
            });
        assert_eq!(res.unwrap(), 42);
    }

    #[test]
    fn sync_fails_after_max() {
        let res: anyhow::Result<()> = Retry::new()
            .max_attempts(1)
            .run_sync(|| anyhow::bail!("nope"));
        assert!(res.is_err());
    }

    #[test]
    fn backoff_caps_at_max_delay() {
        let r = Retry::new().max_delay(Duration::from_secs(2));
        let d = r.next_delay(Duration::from_secs(10));
        assert_eq!(d, Duration::from_secs(2));
    }
}
