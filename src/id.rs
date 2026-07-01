//! ID generation: UUID v4/v7, NanoID, ULID, and Snowflake.
//!
//! Requires the `id` feature.

use std::sync::{LazyLock, Mutex};

use chrono::Utc;

/// Generate a UUID v4 (random) as a hyphenated string.
pub fn uuid_v4() -> String {
    uuid::Uuid::new_v4().to_string()
}

/// Generate a UUID v7 (time-ordered) as a hyphenated string.
pub fn uuid_v7() -> String {
    uuid::Uuid::now_v7().to_string()
}

/// Generate a NanoID of `size` characters from the default URL-safe alphabet.
pub fn nanoid_str(size: usize) -> String {
    nanoid::format(nanoid::rngs::default, &nanoid::alphabet::SAFE, size)
}

/// Generate a default-length (21-char) NanoID.
pub fn nanoid_default() -> String {
    nanoid::format(nanoid::rngs::default, &nanoid::alphabet::SAFE, 21)
}

/// Generate a ULID as a 26-char Crockford base32 string.
pub fn ulid_str() -> String {
    ulid::Ulid::new().to_string()
}

// --- Snowflake (Twitter-style 64-bit ID) ---------------------------------
// | 1 sign | 41 timestamp millis | 10 machine | 12 sequence |

const EPOCH_MILLIS: i64 = 1_704_067_200_000; // 2024-01-01 00:00:00 UTC
const MACHINE_BITS: u8 = 10;
const SEQUENCE_BITS: u8 = 12;
const MAX_SEQUENCE: i64 = (1 << SEQUENCE_BITS) - 1;
const MACHINE_MAX: i64 = (1 << MACHINE_BITS) - 1;

struct SnowflakeState {
    last_ts: i64,
    seq: i64,
}

/// A thread-safe Snowflake ID generator.
pub struct Snowflake {
    machine_id: i64,
    state: Mutex<SnowflakeState>,
}

impl Snowflake {
    /// Create a generator with the given `machine_id` (lower 10 bits used).
    pub fn new(machine_id: u64) -> Self {
        Snowflake {
            machine_id: (machine_id as i64) & MACHINE_MAX,
            state: Mutex::new(SnowflakeState {
                last_ts: -1,
                seq: 0,
            }),
        }
    }

    /// Produce the next monotonic ID.
    pub fn next_id(&self) -> i64 {
        let mut st = self.state.lock().expect("snowflake lock poisoned");
        let mut now = Utc::now().timestamp_millis() - EPOCH_MILLIS;
        if now == st.last_ts {
            st.seq = (st.seq + 1) & MAX_SEQUENCE;
            if st.seq == 0 {
                // sequence exhausted in this ms; wait for next ms
                while now <= st.last_ts {
                    now = Utc::now().timestamp_millis() - EPOCH_MILLIS;
                }
            }
        } else if now < st.last_ts {
            // clock moved backwards; wait until it catches up
            while now < st.last_ts {
                now = Utc::now().timestamp_millis() - EPOCH_MILLIS;
            }
            st.seq = 0;
        } else {
            st.seq = 0;
        }
        st.last_ts = now;
        (now << (MACHINE_BITS + SEQUENCE_BITS)) | (self.machine_id << SEQUENCE_BITS) | st.seq
    }
}

static DEFAULT_SNOWFLAKE: LazyLock<Snowflake> = LazyLock::new(|| Snowflake::new(1));

/// Generate a Snowflake ID from the process-wide default generator.
pub fn snowflake_next() -> i64 {
    DEFAULT_SNOWFLAKE.next_id()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uuid_v4_format() {
        let s = uuid_v4();
        assert_eq!(s.len(), 36);
        // version nibble is '4'
        assert_eq!(s.as_bytes()[14] as char, '4');
    }

    #[test]
    fn uuid_v7_format_and_version() {
        let s = uuid_v7();
        assert_eq!(s.len(), 36);
        let v = s.as_bytes()[14] as char;
        assert_eq!(v, '7');
    }

    #[test]
    fn uuid_are_unique() {
        let a = uuid_v4();
        let b = uuid_v4();
        assert_ne!(a, b);
    }

    #[test]
    fn nanoid_length_and_alphabet() {
        let s = nanoid_str(24);
        assert_eq!(s.len(), 24);
        assert!(
            s.bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
        );
    }

    #[test]
    fn nanoid_default_length() {
        assert_eq!(nanoid_default().len(), 21);
    }

    #[test]
    fn ulid_length_and_alphabet() {
        let s = ulid_str();
        assert_eq!(s.len(), 26);
        assert!(
            s.chars()
                .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit())
        );
    }

    #[test]
    fn snowflake_monotonic_and_unique() {
        let sf = Snowflake::new(7);
        let mut prev = sf.next_id();
        let mut ids = vec![prev];
        for _ in 0..1000 {
            let id = sf.next_id();
            assert!(id > prev, "id {id} not > prev {prev}");
            assert!(!ids.contains(&id), "duplicate id {id}");
            ids.push(id);
            prev = id;
        }
    }

    #[test]
    fn snowflake_encodes_machine_id() {
        let sf = Snowflake::new(123);
        let id = sf.next_id();
        let machine = (id >> SEQUENCE_BITS) & MACHINE_MAX;
        assert_eq!(machine, 123);
    }

    #[test]
    fn snowflake_global_unique() {
        let a = snowflake_next();
        let b = snowflake_next();
        assert_ne!(a, b);
        assert!(b > a);
    }
}
