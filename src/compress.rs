//! Compression helpers: gzip (flate2) and zstd.
//!
//! Requires the `compress` feature.

use std::io::{Read, Write};

use anyhow::Result;
use flate2::{Compression, read::GzDecoder, write::GzEncoder};

/// Compress bytes with gzip.
pub fn gzip_compress(data: &[u8]) -> Result<Vec<u8>> {
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(data)?;
    Ok(encoder.finish()?)
}

/// Decompress gzip bytes.
pub fn gzip_decompress(data: &[u8]) -> Result<Vec<u8>> {
    let mut decoder = GzDecoder::new(data);
    let mut out = Vec::new();
    decoder.read_to_end(&mut out)?;
    Ok(out)
}

/// Compress bytes with zstd at the given level (0–22, 3 is a sane default).
pub fn zstd_compress(data: &[u8], level: i32) -> Result<Vec<u8>> {
    Ok(zstd::encode_all(data, level)?)
}

/// Decompress zstd bytes.
pub fn zstd_decompress(data: &[u8]) -> Result<Vec<u8>> {
    Ok(zstd::decode_all(data)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gzip_roundtrip() {
        let raw = b"lorem ipsum dolor sit amet ".repeat(20);
        let compressed = gzip_compress(&raw).unwrap();
        assert!(compressed.len() < raw.len());
        assert_eq!(gzip_decompress(&compressed).unwrap(), raw);
    }

    #[test]
    fn gzip_empty() {
        let c = gzip_compress(b"").unwrap();
        assert!(gzip_decompress(&c).unwrap().is_empty());
    }

    #[test]
    fn gzip_invalid_input_errors() {
        assert!(gzip_decompress(b"not gzip").is_err());
    }

    #[test]
    fn zstd_roundtrip() {
        let raw = b"repeating repeating repeating ".repeat(30);
        let compressed = zstd_compress(&raw, 3).unwrap();
        assert!(compressed.len() < raw.len());
        assert_eq!(zstd_decompress(&compressed).unwrap(), raw);
    }

    #[test]
    fn zstd_empty() {
        let c = zstd_compress(b"", 3).unwrap();
        assert!(zstd_decompress(&c).unwrap().is_empty());
    }
}
