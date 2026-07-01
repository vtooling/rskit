//! System / process helpers. The `sys` feature gates the parts that depend on
//! `sysinfo`; the raw-pointer and OS-specific helpers are always available.

/// Converts a null-terminated UTF-16 string pointer to a Rust `String`.
///
/// # Safety
///
/// The caller must ensure:
/// 1. `pwstr` is valid for reads up to and including the first `0u16` terminator
///    (or `MAX_LEN` code units if unterminated, in which case `None` is returned);
/// 2. the memory is not mutated concurrently;
/// 3. a non-null `pwstr` is properly aligned.
pub unsafe fn pwstr_to_string(pwstr: *const u16) -> Option<String> {
    if pwstr.is_null() {
        return None;
    }

    let len = {
        let mut len = 0isize;
        const MAX_LEN: isize = 0x10000;

        while len < MAX_LEN {
            if unsafe { *pwstr.offset(len) } == 0 {
                break;
            }
            len += 1;
        }

        if len == MAX_LEN {
            return None;
        }
        len
    };

    let data = unsafe { std::slice::from_raw_parts(pwstr, len as usize) };
    Some(String::from_utf16_lossy(data))
}

/// Check whether a named process is running (Linux only, via `pgrep`).
#[cfg(target_os = "linux")]
pub fn is_running(process: &str) -> bool {
    let output = std::process::Command::new("pgrep").arg(process).output();
    match output {
        Ok(o) => !o.stdout.is_empty(),
        Err(_) => false,
    }
}

/// Detect whether another instance of the current executable is already running.
#[cfg(feature = "sys")]
pub fn is_running_current() -> bool {
    let pid = std::process::id();
    let Ok(exe) = std::env::current_exe() else {
        return false;
    };
    let Some(exe_name) = exe.file_name().and_then(|n| n.to_str()) else {
        return false;
    };
    let system = sysinfo::System::new_all();
    for proc in system.processes().values() {
        if proc.name().to_str() == Some(exe_name) && proc.pid().as_u32() != pid {
            return true;
        }
    }
    false
}

/// Register the current executable to start with Windows (Windows only).
#[cfg(target_os = "windows")]
pub fn set_windows_startup(name: &str) -> anyhow::Result<()> {
    let exe = std::env::current_exe()?;
    let exe_name = exe.to_str().expect("exe path is not utf-8");
    let hkcu = winreg::RegKey::predef(winreg::enums::HKEY_CURRENT_USER);
    let (run, _) = hkcu.create_subkey(r"Software\Microsoft\Windows\CurrentVersion\Run")?;
    run.set_value(name, &exe_name)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pwstr_null_is_none() {
        assert_eq!(unsafe { pwstr_to_string(std::ptr::null()) }, None);
    }

    #[test]
    fn pwstr_empty() {
        let wstr: Vec<u16> = vec![0];
        assert_eq!(
            unsafe { pwstr_to_string(wstr.as_ptr()) },
            Some(String::new())
        );
    }

    #[test]
    fn pwstr_basic() {
        let wstr: Vec<u16> = vec![0x48, 0x65, 0x6C, 0x6C, 0x6F, 0];
        assert_eq!(
            unsafe { pwstr_to_string(wstr.as_ptr()) },
            Some("Hello".to_string())
        );
    }

    #[test]
    fn pwstr_unicode() {
        let wstr: Vec<u16> = vec![0x4F60, 0x597D, 0x4E16, 0x754C, 0];
        assert_eq!(
            unsafe { pwstr_to_string(wstr.as_ptr()) },
            Some("你好世界".to_string())
        );
    }

    #[test]
    fn pwstr_unterminated_returns_none() {
        // No null terminator within MAX_LEN.
        let wstr: Vec<u16> = vec![0x41u16; 0x10000];
        assert_eq!(unsafe { pwstr_to_string(wstr.as_ptr()) }, None);
    }

    #[cfg(feature = "sys")]
    #[test]
    fn is_running_current_is_bool() {
        let _ = is_running_current();
    }
}
