//! Filesystem convenience helpers (std-only, no extra deps).

use std::path::Path;

use anyhow::{Context, Result};

/// Read a file to a `String`.
pub fn read_to_string<P: AsRef<Path>>(path: P) -> Result<String> {
    let p = path.as_ref();
    std::fs::read_to_string(p).with_context(|| format!("read {}", p.display()))
}

/// Read a file to bytes.
pub fn read<P: AsRef<Path>>(path: P) -> Result<Vec<u8>> {
    let p = path.as_ref();
    std::fs::read(p).with_context(|| format!("read {}", p.display()))
}

/// Overwrite a file with bytes (creates or truncates).
pub fn write<P: AsRef<Path>>(path: P, data: &[u8]) -> Result<()> {
    let p = path.as_ref();
    std::fs::write(p, data).with_context(|| format!("write {}", p.display()))?;
    Ok(())
}

/// Overwrite a file with a string.
pub fn write_string<P: AsRef<Path>>(path: P, data: &str) -> Result<()> {
    write(path, data.as_bytes())
}

/// Append bytes to a file, creating it if needed.
pub fn append<P: AsRef<Path>>(path: P, data: &[u8]) -> Result<()> {
    use std::io::Write;
    let p = path.as_ref();
    if let Some(parent) = p.parent().filter(|parent| !parent.as_os_str().is_empty()) {
        std::fs::create_dir_all(parent)?;
    }
    let mut f = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(p)
        .with_context(|| format!("open (append) {}", p.display()))?;
    f.write_all(data)?;
    Ok(())
}

/// Atomically replace a file's contents by writing a temp sibling then renaming.
pub fn atomic_write<P: AsRef<Path>>(path: P, data: &[u8]) -> Result<()> {
    let p = path.as_ref();
    let tmp = p.with_extension(format!(
        "{}.tmp",
        p.extension().and_then(|e| e.to_str()).unwrap_or("rskit")
    ));
    std::fs::write(&tmp, data).with_context(|| format!("write tmp {}", tmp.display()))?;
    std::fs::rename(&tmp, p).with_context(|| format!("rename -> {}", p.display()))?;
    Ok(())
}

/// Create the directory and any parents if missing.
pub fn ensure_dir<P: AsRef<Path>>(path: P) -> Result<()> {
    Ok(std::fs::create_dir_all(path)?)
}

/// Returns `true` if the path exists on disk.
pub fn exists<P: AsRef<Path>>(path: P) -> bool {
    path.as_ref().exists()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp_path(name: &str) -> std::path::PathBuf {
        let mut p = std::env::temp_dir();
        p.push(format!("rskit-fs-{}-{}", std::process::id(), name));
        // clean any pre-existing file
        let _ = std::fs::remove_file(&p);
        p
    }

    #[test]
    fn write_read_roundtrip() {
        let p = tmp_path("wr.txt");
        write_string(&p, "hello").unwrap();
        assert_eq!(read_to_string(&p).unwrap(), "hello");
        std::fs::remove_file(p).unwrap();
    }

    #[test]
    fn read_bytes() {
        let p = tmp_path("bytes.bin");
        write(&p, &[0u8, 1, 2, 3]).unwrap();
        assert_eq!(read(&p).unwrap(), vec![0u8, 1, 2, 3]);
        std::fs::remove_file(p).unwrap();
    }

    #[test]
    fn append_creates_and_grows() {
        let p = tmp_path("append.txt");
        append(&p, b"a").unwrap();
        append(&p, b"b").unwrap();
        assert_eq!(read_to_string(&p).unwrap(), "ab");
        std::fs::remove_file(p).unwrap();
    }

    #[test]
    fn atomic_write_replaces() {
        let p = tmp_path("atomic.txt");
        write_string(&p, "old").unwrap();
        atomic_write(&p, b"new").unwrap();
        assert_eq!(read_to_string(&p).unwrap(), "new");
        std::fs::remove_file(p).unwrap();
    }

    #[test]
    fn ensure_dir_idempotent() {
        let dir = std::env::temp_dir().join(format!("rskit-ensure-{}", std::process::id()));
        ensure_dir(&dir).unwrap();
        ensure_dir(&dir).unwrap();
        assert!(dir.exists());
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn exists_and_missing() {
        let p = tmp_path("exists.txt");
        assert!(!exists(&p));
        write_string(&p, "x").unwrap();
        assert!(exists(&p));
        std::fs::remove_file(p).unwrap();
    }

    #[test]
    fn read_missing_errors() {
        let p = tmp_path("nope.txt");
        assert!(read_to_string(&p).is_err());
    }
}
