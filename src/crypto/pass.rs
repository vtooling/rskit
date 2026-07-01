//! Password hashing with Argon2 and bcrypt, plus a unified verifier.
//!
//! Requires the `pass` feature.

use anyhow::Result;
use argon2::Argon2;
use argon2::password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString};

fn ph_err(e: impl std::fmt::Display) -> anyhow::Error {
    anyhow::anyhow!("{e}")
}

/// Hash a password using Argon2id (random salt). Returns a PHC string.
pub fn argon2_hash(password: &str) -> Result<String> {
    let salt = SaltString::generate(&mut rand::rngs::OsRng);
    let phc = Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map_err(ph_err)?;
    Ok(phc.to_string())
}

/// Verify a password against an Argon2 PHC string.
pub fn argon2_verify(password: &str, phc: &str) -> Result<bool> {
    let parsed = PasswordHash::new(phc).map_err(ph_err)?;
    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed)
        .is_ok())
}

/// Hash a password using bcrypt at the given cost (4–31). Returns a PHC string.
pub fn bcrypt_hash(password: &str, cost: u32) -> Result<String> {
    Ok(bcrypt::hash(password, cost)?)
}

/// Verify a password against a bcrypt PHC string.
pub fn bcrypt_verify(password: &str, phc: &str) -> Result<bool> {
    Ok(bcrypt::verify(password, phc)?)
}

/// Verify a password against a PHC string of either supported algorithm,
/// detected by prefix (`$argon2` / `$2`). Returns `Ok(false)` if the algorithm
/// is unrecognized or the password does not match.
pub fn verify_password(password: &str, phc: &str) -> Result<bool> {
    if phc.starts_with("$argon2") {
        Ok(argon2_verify(password, phc)?)
    } else if phc.starts_with("$2") {
        Ok(bcrypt_verify(password, phc)?)
    } else {
        Ok(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn argon2_roundtrip() {
        let phc = argon2_hash("hunter2").unwrap();
        assert!(phc.starts_with("$argon2id$"));
        assert!(argon2_verify("hunter2", &phc).unwrap());
        assert!(!argon2_verify("wrong", &phc).unwrap());
    }

    #[test]
    fn argon2_unique_salts() {
        let a = argon2_hash("same").unwrap();
        let b = argon2_hash("same").unwrap();
        assert_ne!(a, b);
        assert!(argon2_verify("same", &a).unwrap());
        assert!(argon2_verify("same", &b).unwrap());
    }

    #[test]
    fn bcrypt_roundtrip() {
        let phc = bcrypt_hash("secret", 4).unwrap();
        assert!(phc.starts_with("$2"));
        assert!(bcrypt_verify("secret", &phc).unwrap());
        assert!(!bcrypt_verify("nope", &phc).unwrap());
    }

    #[test]
    fn unified_verifier_detects_algorithm() {
        let argon = argon2_hash("pw").unwrap();
        let bc = bcrypt_hash("pw", 4).unwrap();
        assert!(verify_password("pw", &argon).unwrap());
        assert!(verify_password("pw", &bc).unwrap());
        assert!(!verify_password("pw", "$unknown$xxx").unwrap());
        assert!(!verify_password("wrong", &argon).unwrap());
    }

    #[test]
    fn argon2_malhash_errors() {
        assert!(argon2_verify("pw", "not-a-hash").is_err());
    }
}
