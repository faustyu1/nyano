use std::io;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

/// Create a timestamped backup of the file being saved.
///
/// Fix #5a: each backup gets a unique `.<timestamp>.bak` suffix so earlier
/// backups are not silently overwritten.
pub fn create_backup(path: &Path, saved_text: &str) -> io::Result<()> {
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let mut backup_path = path.as_os_str().to_os_string();
    backup_path.push(format!(".{ts}.bak"));
    std::fs::write(backup_path, saved_text)
}
