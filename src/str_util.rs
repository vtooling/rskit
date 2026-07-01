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

/// Mask the interior of a string, keeping `keep_head` leading and `keep_tail`
/// trailing characters. Strings shorter than `keep_head + keep_tail` are fully
/// masked.
fn mask_inner(s: &str, keep_head: usize, keep_tail: usize) -> String {
    let chars: Vec<char> = s.chars().collect();
    let len = chars.len();
    if len <= keep_head + keep_tail {
        return "*".repeat(len);
    }
    let head: String = chars[..keep_head].iter().collect();
    let tail: String = chars[len - keep_tail..].iter().collect();
    let masked = "*".repeat(len - keep_head - keep_tail);
    format!("{head}{masked}{tail}")
}

/// Mask an email: keep the first char of the local part, mask the rest; keep
/// the domain. `"alice@example.com"` -> `"a***@example.com"`.
pub fn mask_email(s: &str) -> String {
    match s.split_once('@') {
        Some((local, domain)) => format!("{}@{}", mask_inner(local, 1, 0), domain),
        None => mask_inner(s, 1, 0),
    }
}

/// Mask a phone number: keep the first 3 and last 4 digits.
/// `"13812345678"` -> `"138****5678"`.
pub fn mask_phone(s: &str) -> String {
    mask_inner(s, 3, 4)
}

/// Mask a card/account number: keep the first 4 and last 4 digits.
/// `"4242424242424242"` -> `"4242********4242"`.
pub fn mask_card(s: &str) -> String {
    mask_inner(s, 4, 4)
}

/// Levenshtein edit distance between two strings.
pub fn levenshtein(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    let (m, n) = (a.len(), b.len());
    if m == 0 {
        return n;
    }
    if n == 0 {
        return m;
    }
    let mut prev: Vec<usize> = (0..=n).collect();
    let mut curr: Vec<usize> = vec![0; n + 1];
    for i in 1..=m {
        curr[0] = i;
        for j in 1..=n {
            let cost = if a[i - 1] == b[j - 1] { 0 } else { 1 };
            curr[j] = (prev[j] + 1).min(curr[j - 1] + 1).min(prev[j - 1] + cost);
        }
        std::mem::swap(&mut prev, &mut curr);
    }
    prev[n]
}

/// Render a `{key}` template against a variable map. Unknown keys are left
/// verbatim (including braces).
pub fn render_template(tpl: &str, vars: &std::collections::HashMap<&str, String>) -> String {
    let chars: Vec<char> = tpl.chars().collect();
    let mut out = String::with_capacity(tpl.len());
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '{'
            && let Some(offset) = chars[i + 1..].iter().position(|c| *c == '}')
        {
            let end = i + 1 + offset;
            let key: String = chars[i + 1..end].iter().collect();
            match vars.get(key.as_str()) {
                Some(v) => out.push_str(v),
                None => out.push_str(&format!("{{{key}}}")),
            }
            i = end + 1;
            continue;
        }
        out.push(chars[i]);
        i += 1;
    }
    out
}

/// Return `singular` for a count of 1, otherwise `plural`.
pub fn pluralize(count: usize, singular: &str, plural: &str) -> String {
    if count == 1 {
        singular.to_string()
    } else {
        plural.to_string()
    }
}

/// Like [`pluralize`] but auto-appends `"s"` for the plural form.
pub fn pluralize_auto(count: usize, word: &str) -> String {
    if count == 1 {
        word.to_string()
    } else {
        format!("{word}s")
    }
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

    #[test]
    fn mask_helpers() {
        // "alice" (5 chars) keeps 1 leading => 4 stars
        assert_eq!(mask_email("alice@example.com"), "a****@example.com");
        assert_eq!(mask_phone("13812345678"), "138****5678");
        assert_eq!(mask_card("4242424242424242"), "4242********4242");
    }

    #[test]
    fn mask_short_input_fully_masked() {
        assert_eq!(mask_phone("123"), "***");
        assert_eq!(mask_email("x@y.co"), "*@y.co");
    }

    #[test]
    fn levenshtein_cases() {
        assert_eq!(levenshtein("", ""), 0);
        assert_eq!(levenshtein("abc", ""), 3);
        assert_eq!(levenshtein("kitten", "sitting"), 3);
        assert_eq!(levenshtein("flaw", "lawn"), 2);
        assert_eq!(levenshtein("same", "same"), 0);
    }

    #[test]
    fn render_template_substitutes() {
        let mut vars = std::collections::HashMap::new();
        vars.insert("name", "rskit".to_string());
        vars.insert("ver", "0.1".to_string());
        assert_eq!(
            render_template("Hello {name} v{ver}", &vars),
            "Hello rskit v0.1"
        );
    }

    #[test]
    fn render_template_unknown_kept() {
        let vars = std::collections::HashMap::new();
        assert_eq!(render_template("a {missing} b", &vars), "a {missing} b");
        assert_eq!(render_template("no vars here", &vars), "no vars here");
        assert_eq!(render_template("unclosed {key", &vars), "unclosed {key");
    }

    #[test]
    fn pluralize_cases() {
        assert_eq!(pluralize(1, "item", "items"), "item");
        assert_eq!(pluralize(0, "item", "items"), "items");
        assert_eq!(pluralize(5, "item", "items"), "items");
        assert_eq!(pluralize_auto(1, "user"), "user");
        assert_eq!(pluralize_auto(2, "user"), "users");
    }
}
