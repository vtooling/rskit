//! Retry recovers from transient failures and exhausts on permanent ones.

use std::sync::{
    Arc,
    atomic::{AtomicU32, Ordering},
};
use std::time::Duration;

use rskit::retry::Retry;

#[tokio::test(start_paused = true)]
async fn recovers_after_two_failures() {
    let attempts = Arc::new(AtomicU32::new(0));
    let a = attempts.clone();
    let value: u32 = Retry::new()
        .max_attempts(3)
        .initial_delay(Duration::from_millis(10))
        .run_async(move || {
            let a = a.clone();
            async move {
                let n = a.fetch_add(1, Ordering::SeqCst) + 1;
                if n < 3 {
                    anyhow::bail!("transient")
                }
                Ok(99u32)
            }
        })
        .await
        .unwrap();
    assert_eq!(value, 99);
    assert_eq!(attempts.load(Ordering::SeqCst), 3);
}

#[tokio::test(start_paused = true)]
async fn returns_error_after_exhausting() {
    let attempts = Arc::new(AtomicU32::new(0));
    let a = attempts.clone();
    let res: anyhow::Result<u32> = Retry::new()
        .max_attempts(2)
        .initial_delay(Duration::from_millis(5))
        .run_async(move || {
            let a = a.clone();
            async move {
                a.fetch_add(1, Ordering::SeqCst);
                anyhow::bail!("permanent")
            }
        })
        .await;
    assert!(res.is_err());
    assert_eq!(attempts.load(Ordering::SeqCst), 2);
}
