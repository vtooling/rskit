use std::str::FromStr;

use anyhow::Result;
use num_bigint::BigInt;

/// Convert a big-endian byte slice into its decimal `String` representation.
pub fn bytes_to_int(data: &[u8]) -> String {
    BigInt::from_bytes_be(num_bigint::Sign::Plus, data).to_string()
}

/// Convert a decimal `String` back into the original big-endian bytes.
///
/// Returns `None` if `num` is not a valid integer.
pub fn int_to_bytes(num: &str) -> Option<Vec<u8>> {
    BigInt::from_str(num).ok().map(|bi| bi.to_bytes_be().1)
}

/// Convert a big-endian byte slice into a [`BigInt`].
pub fn bytes_to_bigint(data: &[u8]) -> BigInt {
    BigInt::from_bytes_be(num_bigint::Sign::Plus, data)
}

/// Serialize a [`BigInt`] to its decimal string.
pub fn bigint_to_string(n: &BigInt) -> String {
    n.to_string()
}

/// Parse a decimal string into a [`BigInt`].
pub fn parse_bigint(s: &str) -> Result<BigInt> {
    Ok(BigInt::from_str(s)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_text() {
        let raw = b"hello world";
        let num = bytes_to_int(raw);
        assert_eq!(int_to_bytes(&num), Some(raw.to_vec()));
        assert_eq!(
            String::from_utf8(int_to_bytes(&num).unwrap()).unwrap(),
            "hello world"
        );
    }

    #[test]
    fn empty_input_is_zero() {
        // BigInt canonicalizes zero; empty bytes -> "0".
        assert_eq!(bytes_to_int(b""), "0");
    }

    #[test]
    fn int_to_bytes_invalid_returns_none() {
        assert_eq!(int_to_bytes("not a number"), None);
    }

    #[test]
    fn known_value() {
        assert_eq!(bytes_to_int(b"\x01\x00"), "256");
    }

    #[test]
    fn bigint_helpers() {
        let bi = bytes_to_bigint(b"\xff");
        assert_eq!(bigint_to_string(&bi), "255");
        let parsed = parse_bigint("255").unwrap();
        assert_eq!(parsed, bi);
        assert!(parse_bigint("oops").is_err());
    }
}
