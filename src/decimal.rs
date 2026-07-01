//! Decimal / money helpers (re-exports `rust_decimal::Decimal`).
//!
//! Requires the `decimal` feature.

use anyhow::Result;

pub use rust_decimal::{Decimal, prelude::FromStr};

/// Parse a decimal from a string.
pub fn parse(s: &str) -> Result<Decimal> {
    Ok(Decimal::from_str(s)?)
}

/// Construct from an `f64`, returning `None` for NaN/Infinity.
pub fn from_f64(v: f64) -> Option<Decimal> {
    Decimal::from_f64_retain(v)
}

/// Round to `dp` decimal places (banker's rounding).
pub fn round_dp(d: Decimal, dp: u32) -> Decimal {
    d.round_dp(dp)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_ok() {
        let d = parse("123.456").unwrap();
        assert_eq!(d.to_string(), "123.456");
    }

    #[test]
    fn parse_invalid() {
        assert!(parse("not-a-number").is_err());
    }

    #[test]
    fn from_f64_lossless() {
        let d = from_f64(1.5).unwrap();
        assert_eq!(d.to_string(), "1.5");
    }

    #[test]
    fn arithmetic_and_round() {
        let a = parse("1.236").unwrap();
        assert_eq!(round_dp(a, 2).to_string(), "1.24");
        let b = parse("1.234").unwrap();
        assert_eq!(round_dp(b, 2).to_string(), "1.23");
    }

    #[test]
    fn add_two_decimals() {
        let a = parse("0.1").unwrap();
        let b = parse("0.2").unwrap();
        assert_eq!((a + b).to_string(), "0.3");
    }

    #[test]
    fn from_f64_nan_none() {
        assert!(from_f64(f64::NAN).is_none());
        assert!(from_f64(f64::INFINITY).is_none());
    }
}
