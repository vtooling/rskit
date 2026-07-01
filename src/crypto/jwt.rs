//! JSON Web Token (HS256) encode/decode.
//!
//! Requires the `jwt` feature.

use anyhow::Result;
use chrono::{Duration, Utc};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Serialize, de::DeserializeOwned};

/// Minimal claim set: subject (`sub`) and expiry (`exp`, Unix seconds).
/// Embed your own data by adding fields or defining a separate claim struct.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
}

/// Compute an expiry timestamp `secs` seconds from now.
pub fn exp_after(secs: i64) -> usize {
    (Utc::now() + Duration::seconds(secs)).timestamp() as usize
}

/// Encode `claims` as an HS256 JWT signed with `secret`.
pub fn encode_jwt<T: Serialize>(claims: &T, secret: &str) -> Result<String> {
    Ok(encode(
        &Header::default(),
        claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )?)
}

/// Decode and validate (signature + exp) an HS256 JWT, returning the claims.
pub fn decode_jwt<T: DeserializeOwned>(token: &str, secret: &str) -> Result<T> {
    let data = decode::<T>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )?;
    Ok(data.claims)
}

/// Convenience: build a [`Claims`] expiring in `ttl_secs` for subject `sub`.
pub fn claims_for(sub: &str, ttl_secs: i64) -> Claims {
    Claims {
        sub: sub.to_string(),
        exp: exp_after(ttl_secs),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip() {
        let secret = "topsecret";
        let claims = claims_for("user-1", 3600);
        let token = encode_jwt(&claims, secret).unwrap();
        assert!(token.split('.').count() == 3);
        let back: Claims = decode_jwt(&token, secret).unwrap();
        assert_eq!(back, claims);
        assert_eq!(back.sub, "user-1");
    }

    #[test]
    fn wrong_secret_rejected() {
        let token = encode_jwt(&claims_for("u", 60), "k1").unwrap();
        assert!(decode_jwt::<Claims>(&token, "k2").is_err());
    }

    #[test]
    fn expired_token_rejected() {
        // jsonwebtoken's default leeway is 60s; expire well beyond it.
        let claims = Claims {
            sub: "x".into(),
            exp: exp_after(-120), // already expired
        };
        let token = encode_jwt(&claims, "k").unwrap();
        assert!(decode_jwt::<Claims>(&token, "k").is_err());
    }

    #[test]
    fn malformed_token_rejected() {
        assert!(decode_jwt::<Claims>("not.a.jwt", "k").is_err());
    }

    #[test]
    fn exp_after_is_future() {
        let now = Utc::now().timestamp() as usize;
        assert!(exp_after(60) > now);
    }
}
