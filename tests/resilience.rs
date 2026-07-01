//! Rate limiter + circuit breaker protect a flaky downstream.

use std::sync::{
    Arc,
    atomic::{AtomicU32, Ordering},
};
use std::time::Duration;

use rskit::{circuit::CircuitBreaker, rate_limit::TokenBucket};

#[tokio::test]
async fn limiter_gates_calls() {
    let bucket = Arc::new(TokenBucket::new(2, 10.0));
    let hits = Arc::new(AtomicU32::new(0));

    // allow exactly 2 immediately
    for _ in 0..2 {
        assert!(bucket.try_acquire(1));
        hits.fetch_add(1, Ordering::SeqCst);
    }
    assert!(!bucket.try_acquire(1));
    assert_eq!(hits.load(Ordering::SeqCst), 2);
}

#[tokio::test]
async fn breaker_isolates_failures() {
    let cb = Arc::new(CircuitBreaker::new(2, Duration::from_millis(80)));
    let attempts = Arc::new(AtomicU32::new(0));

    for _ in 0..2 {
        let a = attempts.clone();
        let cb = cb.clone();
        let res: anyhow::Result<()> = cb
            .call_async(move || {
                let a = a.clone();
                async move {
                    a.fetch_add(1, Ordering::SeqCst);
                    anyhow::bail!("down")
                }
            })
            .await;
        assert!(res.is_err());
    }
    assert_eq!(cb.state(), rskit::circuit::CircuitState::Open);

    // while open, the closure is NOT even invoked
    let before = attempts.load(Ordering::SeqCst);
    let res: anyhow::Result<()> = cb.call_async(|| async { Ok::<_, anyhow::Error>(()) }).await;
    assert!(res.is_err());
    assert_eq!(attempts.load(Ordering::SeqCst), before);

    // after cooldown, recovery succeeds and resets
    tokio::time::sleep(Duration::from_millis(90)).await;
    let r: i32 = cb
        .call_async(|| async { Ok::<_, anyhow::Error>(7) })
        .await
        .unwrap();
    assert_eq!(r, 7);
    assert_eq!(cb.state(), rskit::circuit::CircuitState::Closed);
}
