//! 安全文件写入：临时文件+flush+fsync+rename；尽力收紧权限。
use crate::security::errors::SecurityError;
use std::fs;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

pub fn secure_atomic_write<P: AsRef<Path>>(
    path: P,
    bytes: &[u8],
) -> Result<PathBuf, SecurityError> {
    let path = path.as_ref();
    let dir = path.parent().unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(dir)?;

    let mut tmp = PathBuf::from(dir);
    tmp.push(format!(
        ".{}.tmp",
        path.file_name().and_then(|s| s.to_str()).unwrap_or("out")
    ));

    let mut f = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(&tmp)
        .map_err(SecurityError::Io)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = fs::set_permissions(&tmp, fs::Permissions::from_mode(0o600));
    }

    f.write_all(bytes).map_err(SecurityError::Io)?;
    f.flush().map_err(SecurityError::Io)?;
    f.sync_all().map_err(SecurityError::Io)?;
    drop(f);
    fs::rename(&tmp, path).map_err(SecurityError::Io)?;

    if let Ok(dirf) = File::open(dir) {
        let _ = dirf.sync_all();
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = fs::set_permissions(path, fs::Permissions::from_mode(0o600));
    }

    // 规范化返回（使用 dunce 避免 Windows 下 \\?\ 前缀）
    dunce::canonicalize(path)
        .or_else(|_| {
            if path.is_absolute() {
                Ok(path.to_path_buf())
            } else {
                std::env::current_dir().map(|cwd| cwd.join(path))
            }
        })
        .map_err(SecurityError::Io)
}
