pub(crate) use {std::path::Path, std::path::PathBuf};
pub(crate) use {std::sync::atomic::AtomicU64, std::sync::atomic::Ordering};

pub(crate) static NEXT_TEMPORARY_ROOT: AtomicU64 = std::sync::atomic::AtomicU64::new(1);

pub(crate) struct TemporaryRoot {
    pub(crate) path: PathBuf,
}

impl TemporaryRoot {
    pub(crate) fn new(label: &str) -> Self {
        let unique = NEXT_TEMPORARY_ROOT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "worth-c9-offline-{label}-{}-{unique}",
            std::process::id()
        ));
        assert!(!path.exists(), "temporary fixture path must be fresh");
        std::fs::create_dir(&path).expect("create temporary fixture root");
        Self { path }
    }

    pub(crate) fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TemporaryRoot {
    fn drop(&mut self) {
        assert!(self.path.starts_with(std::env::temp_dir()));
        let _ = std::fs::remove_dir_all(&self.path);
    }
}