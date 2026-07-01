//! Rate limiting: token bucket (bursty) and leaky bucket (smoothed).

use std::sync::Mutex;
use std::time::{Duration, Instant};

struct TbState {
    tokens: f64,
    last: Instant,
}

/// Token bucket limiter. Allows bursts up to `capacity`, refilling at
/// `refill_per_sec` tokens per second.
pub struct TokenBucket {
    capacity: f64,
    refill_per_sec: f64,
    state: Mutex<TbState>,
}

impl TokenBucket {
    /// `capacity` = maximum tokens (burst size), `refill_per_sec` = refill rate.
    pub fn new(capacity: u32, refill_per_sec: f64) -> Self {
        let cap = capacity as f64;
        TokenBucket {
            capacity: cap,
            refill_per_sec,
            state: Mutex::new(TbState {
                tokens: cap,
                last: Instant::now(),
            }),
        }
    }

    fn refill(&self, st: &mut TbState) {
        let now = Instant::now();
        let elapsed = now.duration_since(st.last).as_secs_f64();
        st.tokens = (st.tokens + elapsed * self.refill_per_sec).min(self.capacity);
        st.last = now;
    }

    /// Try to take `n` tokens without blocking. Returns `false` if not enough.
    pub fn try_acquire(&self, n: u32) -> bool {
        let mut st = self.state.lock().expect("token bucket lock");
        self.refill(&mut st);
        let need = n as f64;
        if st.tokens >= need {
            st.tokens -= need;
            true
        } else {
            false
        }
    }

    /// Block until `n` tokens are available, then take them.
    pub async fn acquire(&self, n: u32) {
        loop {
            let wait = {
                let mut st = self.state.lock().expect("token bucket lock");
                self.refill(&mut st);
                let need = n as f64;
                if st.tokens >= need {
                    st.tokens -= need;
                    return;
                }
                let deficit = need - st.tokens;
                Duration::from_secs_f64(deficit / self.refill_per_sec)
            };
            tokio::time::sleep(wait).await;
        }
    }
}

struct LbState {
    water: f64,
    last: Instant,
}

/// Leaky bucket limiter. Smooths traffic to an average rate: requests add
/// "water"; the bucket leaks at `leak_per_sec`. Overflow is denied.
pub struct LeakyBucket {
    capacity: f64,
    leak_per_sec: f64,
    state: Mutex<LbState>,
}

impl LeakyBucket {
    /// `capacity` = bucket size (queue depth), `leak_per_sec` = drain rate.
    pub fn new(capacity: u32, leak_per_sec: f64) -> Self {
        LeakyBucket {
            capacity: capacity as f64,
            leak_per_sec,
            state: Mutex::new(LbState {
                water: 0.0,
                last: Instant::now(),
            }),
        }
    }

    fn leak(&self, st: &mut LbState) {
        let now = Instant::now();
        let elapsed = now.duration_since(st.last).as_secs_f64();
        st.water = (st.water - elapsed * self.leak_per_sec).max(0.0);
        st.last = now;
    }

    /// Try to add `n` units. Returns `false` if the bucket would overflow.
    pub fn try_acquire(&self, n: u32) -> bool {
        let mut st = self.state.lock().expect("leaky bucket lock");
        self.leak(&mut st);
        let add = n as f64;
        if st.water + add <= self.capacity {
            st.water += add;
            true
        } else {
            false
        }
    }

    /// Block until there is room for `n` units.
    pub async fn acquire(&self, n: u32) {
        loop {
            let wait = {
                let mut st = self.state.lock().expect("leaky bucket lock");
                self.leak(&mut st);
                let add = n as f64;
                if st.water + add <= self.capacity {
                    st.water += add;
                    return;
                }
                let over = st.water + add - self.capacity;
                Duration::from_secs_f64(over / self.leak_per_sec)
            };
            tokio::time::sleep(wait).await;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn token_bucket_burst_then_refill() {
        // capacity 5, refill 5/sec (1 token / 200ms)
        let tb = TokenBucket::new(5, 5.0);
        // burst: can take 5 immediately
        for _ in 0..5 {
            assert!(tb.try_acquire(1));
        }
        // 6th denied
        assert!(!tb.try_acquire(1));
        // after 400ms, ~2 tokens replenished (real sleep)
        tokio::time::sleep(Duration::from_millis(400)).await;
        assert!(tb.try_acquire(1));
        assert!(tb.try_acquire(1));
        assert!(!tb.try_acquire(1));
    }

    #[tokio::test]
    async fn token_bucket_acquire_blocks() {
        let tb = TokenBucket::new(1, 20.0); // 20/sec => ~50ms per token
        assert!(tb.try_acquire(1));
        let start = Instant::now();
        tb.acquire(1).await; // blocks until a token is available
        assert!(start.elapsed() >= Duration::from_millis(30));
    }

    #[tokio::test]
    async fn leaky_bucket_smooths() {
        // capacity 3, leak 10/sec => after burst of 3, ~100ms per unit leaks
        let lb = LeakyBucket::new(3, 10.0);
        assert!(lb.try_acquire(1));
        assert!(lb.try_acquire(1));
        assert!(lb.try_acquire(1));
        assert!(!lb.try_acquire(1)); // full
        tokio::time::sleep(Duration::from_millis(110)).await;
        assert!(lb.try_acquire(1)); // leaked ~1
    }

    #[test]
    fn leaky_bucket_overflow() {
        let lb = LeakyBucket::new(2, 1.0);
        assert!(lb.try_acquire(2));
        assert!(!lb.try_acquire(1));
    }
}
