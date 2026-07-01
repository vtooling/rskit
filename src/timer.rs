use std::str::FromStr;
use std::time::Duration;

use chrono::{DateTime, Local};
use cron::Schedule;

/// Run `f` on every fire of the given cron expression (in local time).
///
/// Blocks the current task forever. Returns immediately (no-op) if the cron
/// expression is invalid.
pub async fn task_cron<F>(cron: &str, f: F)
where
    F: Fn() + Send + 'static,
{
    if let Ok(schedule) = Schedule::from_str(cron) {
        for interval in schedule.upcoming(Local) {
            if let Ok(duration) = interval.signed_duration_since(Local::now()).to_std() {
                tokio::time::sleep(duration).await;
                f();
            }
        }
    }
}

/// Like [`task_cron`] but `f` receives the scheduled fire time.
pub async fn task_cron_with_time<F>(cron: &str, f: F)
where
    F: Fn(DateTime<Local>) + Send + 'static,
{
    if let Ok(schedule) = Schedule::from_str(cron) {
        for interval in schedule.upcoming(Local) {
            if let Ok(duration) = interval.signed_duration_since(Local::now()).to_std() {
                tokio::time::sleep(duration).await;
                f(interval);
            }
        }
    }
}

/// Run `f` repeatedly, waiting `period` between each invocation. Blocks forever.
pub async fn task_interval<F>(period: Duration, f: F)
where
    F: Fn() + Send + 'static,
{
    let mut interval = tokio::time::interval(period);
    loop {
        interval.tick().await;
        f();
    }
}

/// Run `f(&arg)` repeatedly, waiting `period` between each invocation.
pub async fn task_interval_with<F, A>(period: Duration, f: F, arg: A)
where
    F: Fn(&A) + Send + 'static,
    A: Send + 'static,
{
    let mut interval = tokio::time::interval(period);
    loop {
        interval.tick().await;
        f(&arg);
    }
}

/// Run `f` repeatedly starting at `start`, waiting `period` between each.
pub async fn task_interval_at<F>(start: tokio::time::Instant, period: Duration, f: F)
where
    F: Fn() + Send + 'static,
{
    let mut interval = tokio::time::interval_at(start, period);
    loop {
        interval.tick().await;
        f();
    }
}

/// Validate a cron expression without scheduling anything.
pub fn validate_cron(cron: &str) -> bool {
    Schedule::from_str(cron).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{
        Arc,
        atomic::{AtomicU32, Ordering},
    };

    #[test]
    fn validate_cron_ok() {
        assert!(validate_cron("0/5 * * * * *"));
        assert!(validate_cron("0 0 * * * *"));
    }

    #[test]
    fn validate_cron_bad() {
        assert!(!validate_cron("not a cron"));
        assert!(!validate_cron("99 99 99 99 99 99"));
    }

    #[tokio::test(start_paused = true)]
    async fn task_interval_fires_multiple_times() {
        let counter = Arc::new(AtomicU32::new(0));
        let c = counter.clone();

        let handle = tokio::spawn(async move {
            task_interval(Duration::from_millis(10), move || {
                c.fetch_add(1, Ordering::SeqCst);
            })
            .await
        });

        // Advance virtual time enough for several ticks.
        tokio::time::sleep(Duration::from_millis(55)).await;
        handle.abort();

        assert!(counter.load(Ordering::SeqCst) >= 3);
    }

    #[tokio::test(start_paused = true)]
    async fn task_interval_with_passes_arg() {
        let counter = Arc::new(AtomicU32::new(0));
        let c = counter.clone();

        let handle = tokio::spawn(async move {
            task_interval_with(
                Duration::from_millis(10),
                |c: &Arc<AtomicU32>| {
                    c.fetch_add(1, Ordering::SeqCst);
                },
                c,
            )
            .await
        });

        tokio::time::sleep(Duration::from_millis(35)).await;
        handle.abort();

        assert!(counter.load(Ordering::SeqCst) >= 2);
    }
}
