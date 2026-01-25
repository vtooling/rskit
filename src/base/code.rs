use anyhow::Result;
use base58::{FromBase58, ToBase58};
use base64::{Engine, prelude::BASE64_STANDARD};
use sha2::{Digest, Sha256, Sha512};

pub fn base64_encode(s: &[u8]) -> Result<String> {
    Ok(BASE64_STANDARD.encode(s))
}

pub fn base64_decode(s: &str) -> Result<Vec<u8>> {
    let res = BASE64_STANDARD.decode(s)?;
    Ok(res)
}

pub fn base58_encode(s: &[u8]) -> Result<String> {
    Ok(s.to_base58())
}

pub fn base58_decode(s: &str) -> Result<Vec<u8>> {
    let res = s
        .from_base58()
        .map_err(|e| anyhow::anyhow!(format!("{e:?}")))?;
    Ok(res)
}

pub fn sha_256(s: &[u8]) -> String {
    let mut sh = Sha256::new();
    sh.update(s);
    let res = sh.finalize();
    return format!("{:x}", res);
}

pub fn sha_512(s: &[u8]) -> String {
    let mut sh = Sha512::new();
    sh.update(s);
    let res = sh.finalize();
    return format!("{:x}", res);
}

pub fn hash_256(s: &str) -> String {
    let mut sh = Sha256::new();
    sh.update(s);
    let res = sh.finalize();
    return format!("{:x}", res);
}

pub fn hash_512(s: &str) -> String {
    let mut sh = Sha512::new();
    sh.update(s);
    let res = sh.finalize();
    return format!("{:x}", res);
}

/// Converts a null-terminated UTF-16 string pointer to a Rust String.
///
/// # Safety
///
/// This function is unsafe because it dereferences raw pointers. The caller must ensure:
///
/// 1. `pwstr` must be valid for reads up to and including the first null terminator
/// 2. The memory pointed to by `pwstr` must contain a null terminator (0u16) within accessible bounds
/// 3. The pointer must remain valid for the duration of this function call
/// 4. The memory must not be mutated by any other thread during this function call
/// 5. If `pwstr` is non-null, it must be properly aligned (even for empty strings)
///
/// # Undefined Behavior
///
/// Calling this function with a dangling pointer, freed memory, or memory without
/// a null terminator results in undefined behavior.
///
/// # Example
///
/// ```rust
/// use rskit::base::code;
///
/// let wstr: Vec<u16> = vec![0x48, 0x65, 0x6C, 0x6C, 0x6F, 0]; // "Hello\0"
/// let result = unsafe { code::pwstr_to_string(wstr.as_ptr()) };
/// assert_eq!(result, Some("Hello".to_string()));
/// ```
pub unsafe fn pwstr_to_string(pwstr: *const u16) -> Option<String> {
    if pwstr.is_null() {
        return None;
    }

    // Safety: The caller has ensured that pwstr points to valid memory
    // containing a null-terminated UTF-16 string.
    let len = {
        let mut len = 0isize;
        const MAX_LEN: isize = 0x10000; // Prevent infinite loops on malformed input

        while len < MAX_LEN {
            if unsafe { *pwstr.offset(len) } == 0 {
                break;
            }
            len += 1;
        }

        if len == MAX_LEN {
            // No null terminator found within reasonable bounds
            return None;
        }

        len
    };

    let data = unsafe { std::slice::from_raw_parts(pwstr, len as usize) };
    Some(String::from_utf16_lossy(data))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_base64_encode() {
        let s = "hello world";
        let res = base64_encode(s.as_bytes()).unwrap();
        println!("res is: {}", res);
    }

    #[test]
    fn test_base64_decode() {
        let s = "hello world";
        let ss = base64_encode(s.as_bytes()).unwrap();
        let res = base64_decode(&ss).unwrap();
        println!("res is: {}", String::from_utf8(res).unwrap());
    }

    #[test]
    fn test_base58_encode() {
        let s = "hello world";
        let res = base58_encode(s.as_bytes()).unwrap();
        println!("res is: {}", res);
    }

    #[test]
    fn test_base58_decode() {
        let s = "hello world";
        let ss = base58_encode(s.as_bytes()).unwrap();
        let res = base58_decode(&ss).unwrap();
        println!("res is: {}", String::from_utf8(res).unwrap());
    }

    #[test]
    fn test_sh256() {
        let res = hash_256("hello world");
        println!("hash 256 is {}", res);
    }

    #[test]
    fn test_sh512() {
        let res = hash_512("hello world");
        println!("hash 512 is {}", res);
    }

    #[test]
    fn test_pwstr_to_string_null() {
        let result = unsafe { pwstr_to_string(std::ptr::null()) };
        assert_eq!(result, None);
    }

    #[test]
    fn test_pwstr_to_string_empty() {
        let wstr: Vec<u16> = vec![0];
        let result = unsafe { pwstr_to_string(wstr.as_ptr()) };
        assert_eq!(result, Some(String::new()));
    }

    #[test]
    fn test_pwstr_to_string_basic() {
        let wstr: Vec<u16> = vec![0x48, 0x65, 0x6C, 0x6C, 0x6F, 0];
        let result = unsafe { pwstr_to_string(wstr.as_ptr()) };
        assert_eq!(result, Some("Hello".to_string()));
    }

    #[test]
    fn test_pwstr_to_string_unicode() {
        let wstr: Vec<u16> = vec![0x4F60, 0x597D, 0x4E16, 0x754C, 0]; // "你好世界"
        let result = unsafe { pwstr_to_string(wstr.as_ptr()) };
        assert_eq!(result, Some("你好世界".to_string()));
    }

    #[test]
    fn test_pwstr_to_string_with_surrogates() {
        let wstr: Vec<u16> = vec![0xD83D, 0xDE00, 0];
        let result = unsafe { pwstr_to_string(wstr.as_ptr()) };
        assert!(result.is_some());
    }

    #[test]
    fn test_pwstr_to_string_max_len_boundary() {
        // Create a string that approaches MAX_LEN but has null terminator
        let wstr: Vec<u16> = vec![0; 0x10000];
        let wstr: Vec<u16> = wstr.iter().map(|_| 0x41).chain([0].iter().copied()).collect();
        let result = unsafe { pwstr_to_string(wstr.as_ptr()) };
        // Should return None because it hits MAX_LEN before finding null
        assert_eq!(result, None);
    }
}
