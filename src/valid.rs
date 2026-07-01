//! Lightweight format validators (no regex).

/// Validate an email address (non-empty local part, single `@`, domain with a
/// dot and no empty labels). Intentionally pragmatic; for strict RFC 5322 use a
/// dedicated crate.
pub fn is_email(s: &str) -> bool {
    let parts: Vec<&str> = s.split('@').collect();
    if parts.len() != 2 {
        return false;
    }
    let (local, domain) = (parts[0], parts[1]);
    if local.is_empty() || domain.is_empty() {
        return false;
    }
    if local.chars().any(|c| c.is_whitespace()) || domain.chars().any(|c| c.is_whitespace()) {
        return false;
    }
    if local.starts_with('.') || local.ends_with('.') {
        return false;
    }
    if !domain.contains('.') || domain.starts_with('.') || domain.ends_with('.') {
        return false;
    }
    domain.split('.').all(|label| !label.is_empty())
}

/// Validate an `http`/`https` URL (scheme + non-empty host).
pub fn is_url(s: &str) -> bool {
    let lower = s.to_lowercase();
    if !(lower.starts_with("http://") || lower.starts_with("https://")) {
        return false;
    }
    let after_scheme = lower.split_once("://").map(|(_, rest)| rest).unwrap_or("");
    let host = after_scheme
        .split(['/', ':', '?', '#'])
        .next()
        .unwrap_or("");
    !host.is_empty() && host.contains('.')
}

/// Validate an IPv4 address.
pub fn is_ipv4(s: &str) -> bool {
    s.parse::<std::net::Ipv4Addr>().is_ok()
}

/// Validate an IPv6 address.
pub fn is_ipv6(s: &str) -> bool {
    s.parse::<std::net::Ipv6Addr>().is_ok()
}

/// Validate a credit-card number via the Luhn algorithm (digits only, 13–19 long).
pub fn is_credit_card(s: &str) -> bool {
    let digits: String = s.chars().filter(|c| c.is_ascii_digit()).collect();
    if !(13..=19).contains(&digits.len()) {
        return false;
    }
    let mut sum = 0u32;
    let mut alt = false;
    for c in digits.chars().rev() {
        let mut d = c.to_digit(10).unwrap();
        if alt {
            d *= 2;
            if d > 9 {
                d -= 9;
            }
        }
        sum += d;
        alt = !alt;
    }
    sum.is_multiple_of(10)
}

/// Validate a hex string (optionally prefixed with `0x`).
pub fn is_hex(s: &str) -> bool {
    let trimmed = s.strip_prefix("0x").unwrap_or(s);
    !trimmed.is_empty() && trimmed.chars().all(|c| c.is_ascii_hexdigit())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn email_valid() {
        assert!(is_email("a@b.com"));
        assert!(is_email("user.name+tag@sub.example.co"));
    }

    #[test]
    fn email_invalid() {
        assert!(!is_email("plainaddress"));
        assert!(!is_email("@b.com"));
        assert!(!is_email("a@.com"));
        assert!(!is_email("a@b"));
        assert!(!is_email("a@b."));
        assert!(!is_email("a b@b.com"));
        assert!(!is_email(".a@b.com"));
    }

    #[test]
    fn url_valid() {
        assert!(is_url("https://example.com"));
        assert!(is_url("http://example.com:8080/path?x=1"));
    }

    #[test]
    fn url_invalid() {
        assert!(!is_url("ftp://example.com"));
        assert!(!is_url("https://"));
        assert!(!is_url("example.com"));
    }

    #[test]
    fn ipv4_valid() {
        assert!(is_ipv4("192.168.1.1"));
        assert!(is_ipv4("0.0.0.0"));
        assert!(is_ipv4("255.255.255.255"));
    }

    #[test]
    fn ipv4_invalid() {
        assert!(!is_ipv4("256.1.1.1"));
        assert!(!is_ipv4("1.2.3"));
        assert!(!is_ipv4("abc"));
    }

    #[test]
    fn ipv6_valid() {
        assert!(is_ipv6("::1"));
        assert!(is_ipv6("2001:db8::1"));
    }

    #[test]
    fn ipv6_invalid() {
        assert!(!is_ipv6("2001:db8::1::1"));
        assert!(!is_ipv6("not-an-ip"));
    }

    #[test]
    fn credit_card_valid() {
        // 4242 4242 4242 4242 is a well-known Luhn-valid test number.
        assert!(is_credit_card("4242424242424242"));
    }

    #[test]
    fn credit_card_invalid() {
        assert!(!is_credit_card("4242424242424241"));
        assert!(!is_credit_card("123"));
        assert!(!is_credit_card(&"0".repeat(40)));
    }

    #[test]
    fn hex_valid() {
        assert!(is_hex("deadBEEF"));
        assert!(is_hex("0xdeadbeef"));
        assert!(is_hex("00"));
    }

    #[test]
    fn hex_invalid() {
        assert!(!is_hex("xyz"));
        assert!(!is_hex("0x"));
        assert!(!is_hex(""));
    }
}
