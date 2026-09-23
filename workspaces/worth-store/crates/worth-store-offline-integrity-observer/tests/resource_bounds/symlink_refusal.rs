use super::*;

#[test]
fn symlinked_artifact_is_refused_without_reading_its_target() {
    let fixture = clean_store("symlink-bound");
    let root = fixture.roots.join("root-0000000000000001.manifest");
    std::fs::remove_file(&root).unwrap();
    let external = fixture
        .store
        .parent()
        .unwrap()
        .join("external-root.manifest");
    std::fs::write(&external, vec![0x5a; 4096]).unwrap();
    create_file_symlink(&external, &root)
        .expect("hostile symlink fixture must be supported on the admitted test host");
    let report = observe_store(&bounded_request(&fixture, limits(100, 16 * 1024, 8, 0))).unwrap();
    let root_observation = report
        .artifacts()
        .iter()
        .find(|artifact| artifact.family() == PhysicalArtifactFamily::RootManifest)
        .unwrap();
    assert_eq!(
        root_observation.outcome(),
        &OfflineIntegrityOutcome::Indeterminate(
            OfflineIndeterminatePhysicalReason::SymlinkBoundExceeded
        )
    );
    let counters = report.counters();
    assert_eq!(counters.symlinks_refused(), 1);
    assert_eq!(counters.bytes_read(), 286);
    assert_eq!(counters.files_opened(), 11);
    assert_eq!(counters.root_manifest_payload_decoder_entries(), 0);
    assert_eq!(counters.exhausted_bounds(), 1);
}

#[test]
fn escaping_unknown_symlink_is_typed_and_counted() {
    let fixture = clean_store("unknown-symlink");
    let external = fixture.store.parent().unwrap().join("external-unknown");
    std::fs::write(&external, b"outside").unwrap();
    let link = fixture.records.join("mystery.record");
    create_file_symlink(&external, &link)
        .expect("hostile symlink fixture must be supported on the admitted test host");
    let report = observe_store(&bounded_request(&fixture, limits(100, 16 * 1024, 8, 0))).unwrap();
    let unknown = report
        .artifacts()
        .iter()
        .find(|artifact| artifact.relative_path() == "families/records/mystery.record")
        .unwrap();
    assert_eq!(
        unknown.outcome(),
        &OfflineIntegrityOutcome::Indeterminate(
            OfflineIndeterminatePhysicalReason::SymlinkBoundExceeded
        )
    );
    assert_eq!(report.counters().symlinks_refused(), 1);
    assert_eq!(report.counters().exhausted_bounds(), 1);
}

#[test]
fn escaping_root_directory_symlink_is_never_followed() {
    let fixture = clean_store("directory-symlink");
    let external = fixture.store.parent().unwrap().join("external-roots");
    std::fs::rename(&fixture.roots, &external).unwrap();
    create_directory_symlink(&external, &fixture.roots)
        .expect("hostile directory-symlink fixture must be supported on the admitted test host");
    let report = observe_store(&bounded_request(&fixture, limits(100, 16 * 1024, 8, 0))).unwrap();
    let root = report
        .artifacts()
        .iter()
        .find(|artifact| artifact.family() == PhysicalArtifactFamily::RootManifest)
        .unwrap();
    assert_eq!(
        root.outcome(),
        &OfflineIntegrityOutcome::Indeterminate(
            OfflineIndeterminatePhysicalReason::SymlinkBoundExceeded
        )
    );
    assert_eq!(report.counters().symlinks_refused(), 1);
    assert_eq!(report.counters().bytes_read(), 286);
    assert_eq!(report.counters().exhausted_bounds(), 1);
}

#[cfg(unix)]
fn create_file_symlink(target: &Path, link: &Path) -> std::io::Result<()> {
    std::os::unix::fs::symlink(target, link)
}

#[cfg(unix)]
fn create_directory_symlink(target: &Path, link: &Path) -> std::io::Result<()> {
    std::os::unix::fs::symlink(target, link)
}

#[cfg(windows)]
fn create_directory_symlink(target: &Path, link: &Path) -> std::io::Result<()> {
    std::os::windows::fs::symlink_dir(target, link)
}

#[cfg(windows)]
fn create_file_symlink(target: &Path, link: &Path) -> std::io::Result<()> {
    std::os::windows::fs::symlink_file(target, link)
}
