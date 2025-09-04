//! 安全文件写入：临时文件+flush+fsync+rename；尽力收紧权限。
use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

pub fn secure_atomic_write<P: AsRef<Path>>(path: P, bytes: &[u8]) -> io::Result<PathBuf> {
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
        .open(&tmp)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = fs::set_permissions(&tmp, fs::Permissions::from_mode(0o600));
    }

    f.write_all(bytes)?;
    f.flush()?;
    f.sync_all()?;
    drop(f);

    fs::rename(&tmp, path)?;

    if let Ok(dirf) = File::open(dir) {
        let _ = dirf.sync_all();
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = fs::set_permissions(path, fs::Permissions::from_mode(0o600));
    }

    fs::canonicalize(path).or_else(|_| {
        if path.is_absolute() {
            Ok(path.to_path_buf())
        } else {
            std::env::current_dir().map(|cwd| cwd.join(path))
        }
    })
}
