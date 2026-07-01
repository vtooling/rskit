//! HOTP / TOTP one-time-password (RFC 4226 / RFC 6238) using HMAC-SHA1.
//!
//! Requires the `totp` feature (which also depends on the core `crypto::hmac`
//! machinery plus the `sha1` crate).

use chrono::Utc;
use hmac::{Hmac, Mac};
use sha1::Sha1;

type HmacSha1 = Hmac<Sha1>;

/// Compute the raw HOTP value (31-bit truncated) for a counter.
fn hotp_raw(secret: &[u8], counter: u64) -> u32 {
    let mut buf = [0u8; 8];
    buf.copy_from_slice(&counter.to_be_bytes());
    let mut mac = HmacSha1::new_from_slice(secret).expect("any key length is valid for HMAC");
    mac.update(&buf);
    let hash = mac.finalize().into_bytes();
    let offset = (hash[hash.len() - 1] & 0x0f) as usize;
    let bytes = &hash[offset..offset + 4];
    u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]) & 0x7fffffff
}

/// RFC 4226 HOTP: produce a `digits`-length zero-padded code.
pub fn hotp(secret: &[u8], counter: u64, digits: u32) -> String {
    let raw = hotp_raw(secret, counter);
    let modulus = 10u32.pow(digits);
    format!(
        "{:0>width$}",
        (raw % modulus) as u64,
        width = digits as usize
    )
}

/// RFC 6238 TOTP at a specific unix `time` (seconds), `step` seconds per window.
pub fn totp(secret: &[u8], time: u64, step: u32, digits: u32) -> String {
    let counter = time / step as u64;
    hotp(secret, counter, digits)
}

/// TOTP for the current time.
pub fn totp_now(secret: &[u8], step: u32, digits: u32) -> String {
    totp(secret, Utc::now().timestamp() as u64, step, digits)
}

/// Verify a TOTP code allowing ±`window` time steps of clock drift.
pub fn totp_verify(
    secret: &[u8],
    time: u64,
    step: u32,
    digits: u32,
    code: &str,
    window: u32,
) -> bool {
    let base = (time / step as u64) as i64;
    for offset in -(window as i64)..=(window as i64) {
        let candidate = base + offset;
        if candidate >= 0 && hotp(secret, candidate as u64, digits) == code {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    // RFC 6238 Appendix B SHA-1 test vectors; secret = "12345678901234567890" (ASCII).
    const SECRET: &[u8] = b"12345678901234567890";

    #[test]
    fn rfc6238_vectors_8_digits() {
        assert_eq!(totp(SECRET, 59, 30, 8), "94287082");
        assert_eq!(totp(SECRET, 1_111_111_109, 30, 8), "07081804");
        assert_eq!(totp(SECRET, 1_111_111_111, 30, 8), "14050471");
        assert_eq!(totp(SECRET, 1_234_567_890, 30, 8), "89005924");
    }

    #[test]
    fn six_digit_mode() {
        assert_eq!(totp(SECRET, 59, 30, 6), "287082");
    }

    #[test]
    fn code_length_respected() {
        let code = totp(SECRET, 59, 30, 6);
        assert_eq!(code.len(), 6);
    }

    #[test]
    fn hotp_is_dynamic_truncation_length() {
        let code = hotp(SECRET, 0, 6);
        assert_eq!(code.len(), 6);
    }

    #[test]
    fn verify_accepts_current_code() {
        let code = totp(SECRET, 1_000_000, 30, 6);
        assert!(totp_verify(SECRET, 1_000_000, 30, 6, &code, 1));
    }

    #[test]
    fn verify_accepts_adjacent_window() {
        // code computed at T=1_000_030 should verify at T=1_000_000 with window=1
        let code = totp(SECRET, 1_000_030, 30, 6);
        assert!(totp_verify(SECRET, 1_000_000, 30, 6, &code, 1));
    }

    #[test]
    fn verify_rejects_out_of_window() {
        let code = totp(SECRET, 1_000_000, 30, 6);
        // far in the future
        assert!(!totp_verify(SECRET, 2_000_000, 30, 6, &code, 1));
        // garbage
        assert!(!totp_verify(SECRET, 1_000_000, 30, 6, "000000", 0));
    }
}
