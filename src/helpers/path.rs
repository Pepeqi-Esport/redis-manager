use std::path::{Path, PathBuf};

pub fn normalize_path(path: &Path) -> PathBuf {
    path.canonicalize()
        .unwrap_or_else(|_| path.to_path_buf())
}

pub fn path_for_windows(path: &Path) -> String {
    let mut s = path.display().to_string().replace('/', "\\");
    // canonicalize() on Windows may prefix \\?\ which can break Run entries.
    if let Some(stripped) = s.strip_prefix(r"\\?\") {
        s = stripped.to_string();
    }
    s
}
