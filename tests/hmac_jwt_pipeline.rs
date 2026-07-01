//! HMAC-signed JWT pipeline.

#![cfg(feature = "jwt")]

use rskit::crypto::{hmac, jwt};

#[test]
fn sign_and_verify_jwt_with_hmac_secret() {
    let secret = b"shared-hmac-secret";
    // A server signs a JWT; the signing key material is itself an HMAC tag of
    // some payload, tying the token to a request fingerprint.
    let fingerprint = hmac::hmac_sha256_hex(secret, b"issuer|rskit");
    let claims = jwt::Claims {
        sub: format!("user:{}", fingerprint),
        exp: jwt::exp_after(3600),
    };
    let token = jwt::encode_jwt(&claims, &fingerprint).unwrap();
    let back: jwt::Claims = jwt::decode_jwt(&token, &fingerprint).unwrap();
    assert_eq!(back.sub, claims.sub);
    assert!(jwt::decode_jwt::<jwt::Claims>(&token, "wrong-secret").is_err());
}
