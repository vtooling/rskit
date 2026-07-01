//! Constant-time comparisons for secrets (uses `subtle`).

use subtle::ConstantTimeEq;

/// Constant-time byte-slice equality. Returns `false` immediately for unequal
/// lengths (length is not secret), then compares contents in constant time.
pub fn ct_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    a.ct_eq(b).into()
}

/// Constant-time comparison of two hex digests (case-insensitive on input is
/// the caller's responsibility; this compares the exact bytes).
pub fn ct_eq_hex(a: &str, b: &str) -> bool {
    match (hex::decode(a), hex::decode(b)) {
        (Ok(da), Ok(db)) => ct_eq(&da, &db),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn equal_slices() {
        assert!(ct_eq(b"secret", b"secret"));
    }

    #[test]
    fn unequal_slices() {
        assert!(!ct_eq(b"secret", b"secre7"));
    }

    #[test]
    fn different_lengths() {
        assert!(!ct_eq(b"abc", b"abcd"));
        assert!(!ct_eq(b"", b"a"));
    }

    #[test]
    fn empty_equal() {
        assert!(ct_eq(b"", b""));
    }

    #[test]
    fn hex_equal_and_not() {
        assert!(ct_eq_hex("deadbeef", "deadbeef"));
        assert!(!ct_eq_hex("deadbeef", "deadbeef00"));
        assert!(!ct_eq_hex("deadbeef", "feedface"));
        assert!(!ct_eq_hex("nothex", "deadbeef"));
    }
}
