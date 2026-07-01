//! Circuit breaker: stop calling a failing dependency after a threshold, and
//! retry after a cooldown.

use std::sync::Mutex;
use std::time::{Duration, Instant};

use anyhow::Result;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitState {
    Closed,
    Open,
    HalfOpen,
}

struct Inner {
    state: CircuitState,
    failures: u32,
    opened_at: Option<Instant>,
}

/// A simple circuit breaker.
///
/// - `Closed`: calls pass through; on `Err`, the failure count rises.
/// - `Open` (after `failure_threshold` consecutive failures): calls short-circuit
///   with an error, until `reset_after` elapses.
/// - `HalfOpen`: the next call is a probe; success closes the circuit, failure
///   re-opens it.
pub struct CircuitBreaker {
    failure_threshold: u32,
    reset_after: Duration,
    inner: Mutex<Inner>,
}

#[derive(Debug)]
pub struct CircuitOpen;

impl std::fmt::Display for CircuitOpen {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "circuit breaker is open")
    }
}

impl std::error::Error for CircuitOpen {}

impl CircuitBreaker {
    pub fn new(failure_threshold: u32, reset_after: Duration) -> Self {
        CircuitBreaker {
            failure_threshold,
            reset_after,
            inner: Mutex::new(Inner {
                state: CircuitState::Closed,
                failures: 0,
                opened_at: None,
            }),
        }
    }

    /// Current state.
    pub fn state(&self) -> CircuitState {
        self.inner.lock().expect("circuit lock").state
    }

    fn allow(&self, st: &mut Inner) -> bool {
        match st.state {
            CircuitState::Closed => true,
            CircuitState::HalfOpen => true,
            CircuitState::Open => {
                if let Some(opened) = st.opened_at
                    && opened.elapsed() >= self.reset_after
                {
                    st.state = CircuitState::HalfOpen;
                    return true;
                }
                false
            }
        }
    }

    /// Run `f`, applying the breaker. Returns an error if open.
    pub async fn call_async<F, Fut, T>(&self, f: F) -> Result<T>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<T>>,
    {
        {
            let mut st = self.inner.lock().expect("circuit lock");
            if !self.allow(&mut st) {
                return Err(CircuitOpen.into());
            }
        }
        let res = f().await;
        let mut st = self.inner.lock().expect("circuit lock");
        match res {
            Ok(v) => {
                st.failures = 0;
                st.state = CircuitState::Closed;
                st.opened_at = None;
                Ok(v)
            }
            Err(e) => {
                st.failures += 1;
                if st.failures >= self.failure_threshold || st.state == CircuitState::HalfOpen {
                    st.state = CircuitState::Open;
                    st.opened_at = Some(Instant::now());
                }
                Err(e)
            }
        }
    }

    /// Synchronous variant.
    pub fn call_sync<F, T>(&self, f: F) -> Result<T>
    where
        F: FnOnce() -> Result<T>,
    {
        {
            let mut st = self.inner.lock().expect("circuit lock");
            if !self.allow(&mut st) {
                return Err(CircuitOpen.into());
            }
        }
        let res = f();
        let mut st = self.inner.lock().expect("circuit lock");
        match res {
            Ok(v) => {
                st.failures = 0;
                st.state = CircuitState::Closed;
                st.opened_at = None;
                Ok(v)
            }
            Err(e) => {
                st.failures += 1;
                if st.failures >= self.failure_threshold || st.state == CircuitState::HalfOpen {
                    st.state = CircuitState::Open;
                    st.opened_at = Some(Instant::now());
                }
                Err(e)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn opens_after_threshold_then_half_open() {
        let cb = CircuitBreaker::new(2, Duration::from_millis(100));
        assert_eq!(cb.state(), CircuitState::Closed);

        let fail: anyhow::Result<()> = cb.call_async(|| async { anyhow::bail!("x") }).await;
        assert!(fail.is_err());
        assert_eq!(cb.state(), CircuitState::Closed); // 1 failure

        let fail: anyhow::Result<()> = cb.call_async(|| async { anyhow::bail!("x") }).await;
        assert!(fail.is_err());
        assert_eq!(cb.state(), CircuitState::Open); // threshold reached

        // open: short-circuited with CircuitOpen
        let err = cb
            .call_async(|| async { Ok::<_, anyhow::Error>(1) })
            .await
            .unwrap_err();
        assert!(err.is::<CircuitOpen>());

        // after reset_after (real sleep): half-open, success closes
        tokio::time::sleep(Duration::from_millis(120)).await;
        let r: i32 = cb
            .call_async(|| async { Ok::<_, anyhow::Error>(42) })
            .await
            .unwrap();
        assert_eq!(r, 42);
        assert_eq!(cb.state(), CircuitState::Closed);
    }

    #[tokio::test]
    async fn half_open_failure_reopens() {
        let cb = CircuitBreaker::new(1, Duration::from_millis(60));
        let fail: anyhow::Result<()> = cb.call_async(|| async { anyhow::bail!("x") }).await;
        assert!(fail.is_err());
        assert_eq!(cb.state(), CircuitState::Open);

        tokio::time::sleep(Duration::from_millis(70)).await;
        // probe fails => reopen
        let fail: anyhow::Result<()> = cb.call_async(|| async { anyhow::bail!("x") }).await;
        assert!(fail.is_err());
        assert_eq!(cb.state(), CircuitState::Open);
    }

    #[test]
    fn sync_closes_on_success() {
        let cb = CircuitBreaker::new(3, Duration::from_secs(1));
        assert!(
            cb.call_sync(|| -> anyhow::Result<()> { anyhow::bail!("x") })
                .is_err()
        );
        assert!(
            cb.call_sync(|| -> anyhow::Result<()> { anyhow::bail!("x") })
                .is_err()
        );
        assert_eq!(cb.state(), CircuitState::Closed); // not yet threshold
        assert!(cb.call_sync(|| Ok::<_, anyhow::Error>(7)).is_ok());
        assert_eq!(cb.state(), CircuitState::Closed);
        // success resets the failure counter
        assert!(
            cb.call_sync(|| -> anyhow::Result<()> { anyhow::bail!("x") })
                .is_err()
        );
        assert!(
            cb.call_sync(|| -> anyhow::Result<()> { anyhow::bail!("x") })
                .is_err()
        );
        assert_eq!(cb.state(), CircuitState::Closed);
    }
}
