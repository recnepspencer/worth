pub(crate) use {
    crate::resource_bounds::bounded_request, crate::resource_bounds::limits,
    crate::support::fixtures::clean_store, std::path::Path,
    worth_foundational::PhysicalArtifactFamily,
    worth_store_offline_integrity_observer::observe_store,
    worth_store_offline_integrity_observer::OfflineIndeterminatePhysicalReason,
    worth_store_offline_integrity_observer::OfflineIntegrityOutcome,
};

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
