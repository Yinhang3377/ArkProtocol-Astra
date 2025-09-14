//! Secure atomic write: write to a temporary file, flush+fsync, rename, then
//! fsync the parent directory. Try to tighten permissions on Unix.
//!
//! This implementation creates a unique temporary filename in the target
//! directory (using OS RNG for a suffix), writes the contents, flushes and
//! syncs the file, then renames it into place and attempts to sync the
//! containing directory. On Unix it tightens permissions to 0o600.

use crate::security::errors::SecurityError;
use dunce;
use rand::rngs::OsRng;
use rand::RngCore;
use std::fs;
use std::fs::{ File, OpenOptions };
use std::io::Write;
use std::path::{ Path, PathBuf };

/// Atomically write `bytes` to `path` and return a canonicalized PathBuf.
/// Maps IO errors to `SecurityError::Io`.
pub fn secure_atomic_write<P: AsRef<Path>>(
    path: P,
    bytes: &[u8]
) -> Result<PathBuf, SecurityError> {
    let path = path.as_ref();
    let dir = path.parent().unwrap_or_else(|| Path::new("."));

    // Ensure the directory exists
    fs::create_dir_all(dir).map_err(SecurityError::Io)?;

    // Build a reasonably unique temporary filename inside the target directory.
    // Using create_new below prevents races with existing files with same name.
    let base = path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("out");

    let mut tmp = PathBuf::from(dir);
    // random suffix to avoid collisions
    let mut rnd = [0u8; 16];
    OsRng.fill_bytes(&mut rnd);
    let suffix = hex::encode(rnd);
    tmp.push(format!(".{}.tmp.{}", base, suffix));

    // Open with create_new so we don't clobber an existing file accidentally.
    let mut f = match OpenOptions::new().write(true).create_new(true).open(&tmp) {
        Ok(h) => h,
        Err(e) => {
            return Err(SecurityError::Io(e));
        }
    };

    // Tighten permissions on Unix for the temporary file.
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = fs::set_permissions(&tmp, fs::Permissions::from_mode(0o600));
    }

    // Write, flush and sync the file to ensure contents are on disk
    if let Err(e) = f.write_all(bytes) {
        let _ = fs::remove_file(&tmp);
        return Err(SecurityError::Io(e));
    }
    if let Err(e) = f.flush() {
        let _ = fs::remove_file(&tmp);
        return Err(SecurityError::Io(e));
    }
    if let Err(e) = f.sync_all() {
        let _ = fs::remove_file(&tmp);
        return Err(SecurityError::Io(e));
    }

    // Close file before renaming
    drop(f);

    // Rename into place (atomic on most platforms if same filesystem)
    if let Err(e) = fs::rename(&tmp, path) {
        let _ = fs::remove_file(&tmp);
        return Err(SecurityError::Io(e));
    }

    // Try to sync the parent directory. This may fail on some platforms; ignore
    // errors but attempt it when possible.
    let _ = File::open(dir).and_then(|d| d.sync_all());

    // Tighten permissions on the final file on Unix as well.
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = fs::set_permissions(path, fs::Permissions::from_mode(0o600));
    }

    // Canonicalize for consistent absolute path; on Windows `dunce::canonicalize`
    // avoids the "\\?\\" prefix.
    dunce
        ::canonicalize(path)
        .or_else(|_| {
            if path.is_absolute() {
                Ok(path.to_path_buf())
            } else {
                std::env::current_dir().map(|cwd| cwd.join(path))
            }
        })
        .map_err(SecurityError::Io)
}
