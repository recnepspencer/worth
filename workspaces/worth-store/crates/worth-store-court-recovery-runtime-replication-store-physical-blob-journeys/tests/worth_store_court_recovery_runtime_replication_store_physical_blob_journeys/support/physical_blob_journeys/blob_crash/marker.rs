use std::path::{Path, PathBuf};

const MARKER: &str = "blob-crash-ready";

pub(crate) fn park_at_marker(root: &Path, object: [u8; 16], session: [u8; 16]) -> ! {
    write_marker(&marker_path(root), object, session);
    loop {
        std::thread::park();
    }
}

pub(crate) fn marker_path(root: &Path) -> PathBuf {
    root.parent().unwrap().join(MARKER)
}

pub(crate) fn resume_token_path(root: &Path) -> PathBuf {
    root.parent().unwrap().join("blob-crash-resume-token")
}

pub(crate) fn write_marker(path: &Path, object: [u8; 16], session: [u8; 16]) {
    let mut identity = [0_u8; 32];
    identity[..16].copy_from_slice(&object);
    identity[16..].copy_from_slice(&session);
    let pending = path.with_extension("pending");
    std::fs::write(&pending, identity).unwrap();
    std::fs::rename(pending, path).unwrap();
}
