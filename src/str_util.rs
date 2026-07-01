//! String utilities: random generation, case conversion, slugify.

use rand::Rng;

const ALPHANUMERIC: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
const HEX_LOWER: &[u8] = b"0123456789abcdef";

/// Generate a random alphanumeric string of length `len` (A-Za-z0-9).
pub fn random_alphanumeric(len: usize) -> String {
    let mut rng = rand::thread_rng();
    (0..len)
        .map(|_| {
            let idx = rng.gen_range(0..ALPHANUMERIC.len());
            ALPHANUMERIC[idx] as char
        })
        .collect()
}

/// Generate a random lowercase-hex string of length `len`.
pub fn random_hex(len: usize) -> String {
    let mut rng = rand::thread_rng();
    (0..len)
        .map(|_| {
            let idx = rng.gen_range(0..HEX_LOWER.len());
            HEX_LOWER[idx] as char
        })
        .collect()
}

/// Generate a random string of length `len` using characters from `charset`.
/// Returns an empty string if `charset` is empty.
pub fn random_string(len: usize, charset: &[char]) -> String {
    if charset.is_empty() {
        return String::new();
    }
    let mut rng = rand::thread_rng();
    (0..len)
        .map(|_| {
            let idx = rng.gen_range(0..charset.len());
            charset[idx]
        })
        .collect()
}

fn split_words(s: &str) -> Vec<String> {
    let mut words: Vec<String> = Vec::new();
    let mut cur = String::new();
    let mut prev_upper = false;

    for ch in s.chars() {
        if !ch.is_alphanumeric() {
            if !cur.is_empty() {
                words.push(std::mem::take(&mut cur));
            }
            prev_upper = false;
            continue;
        }
        let is_upper = ch.is_uppercase();
        if !cur.is_empty() {
            if !prev_upper && is_upper {
                // lower -> upper boundary: camelCase
                words.push(std::mem::take(&mut cur));
            } else if prev_upper && is_upper {
                // run of uppercase continues; wait
            } else if prev_upper && !is_upper && cur.len() > 1 {
                // upper -> lower boundary after an acronym: HTTPServer
                let last = cur.pop().unwrap();
                words.push(cur.clone());
                cur.clear();
                cur.push(last);
            }
        }
        cur.push(ch);
        prev_upper = is_upper;
    }
    if !cur.is_empty() {
        words.push(cur);
    }
    words
}

fn capitalize(word: &str) -> String {
    let mut out = String::new();
    let mut chars = word.chars();
    if let Some(first) = chars.next() {
        for u in first.to_uppercase() {
            out.push(u);
        }
    }
    for c in chars {
        for d in c.to_lowercase() {
            out.push(d);
        }
    }
    out
}

/// Convert to `snake_case`.
pub fn to_snake_case(s: &str) -> String {
    split_words(s)
        .iter()
        .map(|w| w.to_lowercase())
        .collect::<Vec<_>>()
        .join("_")
}

/// Convert to `kebab-case`.
pub fn to_kebab_case(s: &str) -> String {
    split_words(s)
        .iter()
        .map(|w| w.to_lowercase())
        .collect::<Vec<_>>()
        .join("-")
}

/// Convert to `PascalCase`.
pub fn to_pascal_case(s: &str) -> String {
    split_words(s).iter().map(|w| capitalize(w)).collect()
}

/// Convert to `camelCase`.
pub fn to_camel_case(s: &str) -> String {
    let words = split_words(s);
    words
        .iter()
        .enumerate()
        .map(|(i, w)| {
            if i == 0 {
                w.to_lowercase()
            } else {
                capitalize(w)
            }
        })
        .collect()
}

/// Convert to a URL slug: lowercase, ASCII non-alphanumerics collapsed to `-`,
/// trimmed. Non-ASCII characters are treated as separators.
pub fn slugify(s: &str) -> String {
    let mut out = String::new();
    let mut prev_dash = false;
    for ch in s.trim().to_lowercase().chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch);
            prev_dash = false;
        } else if !prev_dash {
            out.push('-');
            prev_dash = true;
        }
    }
    out.trim_matches('-').to_string()
}

/// Truncate `s` to at most `max` chars, appending `suffix` if truncation occurred.
pub fn truncate(s: &str, max: usize, suffix: &str) -> String {
    if s.chars().count() <= max {
        return s.to_string();
    }
    let head: String = s
        .chars()
        .take(max.saturating_sub(suffix.chars().count()))
        .collect();
    format!("{head}{suffix}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn random_alphanumeric_length_and_charset() {
        let s = random_alphanumeric(32);
        assert_eq!(s.len(), 32);
        assert!(s.chars().all(|c| c.is_ascii_alphanumeric()));
    }

    #[test]
    fn random_hex_is_lowercase_hex() {
        let s = random_hex(16);
        assert_eq!(s.len(), 16);
        assert!(
            s.chars()
                .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase())
        );
    }

    #[test]
    fn random_string_with_charset() {
        let charset = ['x', 'y'];
        let s = random_string(50, &charset);
        assert_eq!(s.len(), 50);
        assert!(s.chars().all(|c| c == 'x' || c == 'y'));
    }

    #[test]
    fn random_string_empty_charset() {
        assert_eq!(random_string(10, &[]), "");
    }

    #[test]
    fn case_conversions() {
        assert_eq!(to_snake_case("HelloWorld"), "hello_world");
        assert_eq!(to_snake_case("HTTPServer"), "http_server");
        assert_eq!(to_snake_case("hello world"), "hello_world");
        assert_eq!(to_kebab_case("HelloWorld"), "hello-world");
        assert_eq!(to_pascal_case("hello world foo"), "HelloWorldFoo");
        assert_eq!(to_camel_case("hello world foo"), "helloWorldFoo");
        assert_eq!(to_camel_case("HTTPServer"), "httpServer");
        assert_eq!(to_pascal_case("user_id"), "UserId");
    }

    #[test]
    fn case_empty_and_single() {
        assert_eq!(to_snake_case(""), "");
        assert_eq!(to_pascal_case("x"), "X");
        assert_eq!(to_camel_case("X"), "x");
    }

    #[test]
    fn slugify_examples() {
        assert_eq!(slugify("Hello, World!"), "hello-world");
        assert_eq!(slugify("  rskit   kit  "), "rskit-kit");
        assert_eq!(slugify("你好—world"), "world");
    }

    #[test]
    fn truncate_behavior() {
        assert_eq!(truncate("hello world", 11, "..."), "hello world");
        assert_eq!(truncate("hello world", 8, "..."), "hello...");
        assert_eq!(truncate("abcdef", 3, ""), "abc");
    }
}
