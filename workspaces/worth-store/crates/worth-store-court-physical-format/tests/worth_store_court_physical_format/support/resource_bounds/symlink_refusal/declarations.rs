pub(crate) use {worth_store_offline_integrity_observer::OfflineIndeterminatePhysicalReason, worth_store_offline_integrity_observer::OfflineIntegrityOutcome, std::path::Path, worth_foundational::PhysicalArtifactFamily, crate::resource_bounds::bounded_request, crate::support::fixtures::clean_store, crate::resource_bounds::limits, worth_store_offline_integrity_observer::observe_store};

#[cfg(unix)]
pub(crate) fn create_file_symlink(target: &Path, link: &Path) -> std::io::Result<()> {
    std::os::unix::fs::symlink(target, link)
}

#[cfg(unix)]
pub(crate) fn create_directory_symlink(target: &Path, link: &Path) -> std::io::Result<()> {
    std::os::unix::fs::symlink(target, link)
}

#[cfg(windows)]
pub(crate) fn create_directory_symlink(target: &Path, link: &Path) -> std::io::Result<()> {
    std::os::windows::fs::symlink_dir(target, link)
}

#[cfg(windows)]
pub(crate) fn create_file_symlink(target: &Path, link: &Path) -> std::io::Result<()> {
    std::os::windows::fs::symlink_file(target, link)
}