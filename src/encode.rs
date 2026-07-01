use anyhow::Result;
use base58::{FromBase58, ToBase58};
use base64::{Engine, prelude::BASE64_STANDARD};
use percent_encoding::{NON_ALPHANUMERIC, percent_decode_str, utf8_percent_encode};

/// Encode bytes into a base64 string.
pub fn base64_encode(input: &[u8]) -> String {
    BASE64_STANDARD.encode(input)
}

/// Decode a base64 string into bytes.
pub fn base64_decode(input: &str) -> Result<Vec<u8>> {
    Ok(BASE64_STANDARD.decode(input)?)
}

/// Encode bytes into a base58 string.
pub fn base58_encode(input: &[u8]) -> String {
    input.to_base58()
}

/// Decode a base58 string into bytes.
pub fn base58_decode(input: &str) -> Result<Vec<u8>> {
    input
        .from_base58()
        .map_err(|e| anyhow::anyhow!("base58 decode error: {e:?}"))
}

/// Encode bytes into a lowercase hex string.
pub fn hex_encode(input: &[u8]) -> String {
    hex::encode(input)
}

/// Decode a hex string into bytes.
pub fn hex_decode(input: &str) -> Result<Vec<u8>> {
    Ok(hex::decode(input)?)
}

/// Percent-encode a string for safe use inside a URL query/path.
///
/// All non-alphanumeric characters are encoded.
pub fn url_encode(input: &str) -> String {
    utf8_percent_encode(input, NON_ALPHANUMERIC).to_string()
}

/// Percent-decode a string.
pub fn url_decode(input: &str) -> String {
    percent_decode_str(input).decode_utf8_lossy().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base64_roundtrip() {
        let raw = b"hello world";
        let enc = base64_encode(raw);
        assert_eq!(enc, "aGVsbG8gd29ybGQ=");
        assert_eq!(base64_decode(&enc).unwrap(), raw);
    }

    #[test]
    fn base64_empty() {
        let enc = base64_encode(b"");
        assert_eq!(enc, "");
        assert_eq!(base64_decode(&enc).unwrap(), Vec::<u8>::new());
    }

    #[test]
    fn base58_roundtrip() {
        let raw = b"hello world";
        let enc = base58_encode(raw);
        assert_eq!(enc, "StV1DL6CwTryKyV");
        assert_eq!(base58_decode(&enc).unwrap(), raw);
    }

    #[test]
    fn base58_decode_invalid() {
        assert!(base58_decode("0OIl").is_err());
    }

    #[test]
    fn hex_roundtrip() {
        let raw = b"hello";
        let enc = hex_encode(raw);
        assert_eq!(enc, "68656c6c6f");
        assert_eq!(hex_decode(&enc).unwrap(), raw);
    }

    #[test]
    fn hex_uppercase_input_decoded() {
        assert_eq!(hex_decode("4A4B").unwrap(), vec![0x4a, 0x4b]);
    }

    #[test]
    fn hex_decode_invalid() {
        assert!(hex_decode("nothex").is_err());
    }

    #[test]
    fn url_roundtrip() {
        let raw = "https://example.com/s?wd=ok&msg=你好";
        let enc = url_encode(raw);
        assert_ne!(enc, raw);
        assert_eq!(url_decode(&enc), raw);
    }

    #[test]
    fn url_encode_alphanumeric_unchanged() {
        assert_eq!(url_encode("abc123"), "abc123");
    }

    #[test]
    fn url_decode_literal_plus_kept() {
        // percent_decode does not convert '+' to space; verify behavior is stable.
        assert_eq!(url_decode("a+b"), "a+b");
    }
}
