use std::path::Path;

use worth_foundational::PhysicalArtifactFamily;
use worth_store_offline_integrity_observer::{
    observe_store, OfflineArtifactDuplicateEvidence, OfflineArtifactFamily,
    OfflineIndeterminatePhysicalReason, OfflineIntegrityObservationDenial,
    OfflineIntegrityObservationLimits, OfflineIntegrityObservationRequest, OfflineIntegrityOutcome,
    OfflineIntegrityProtocolContext, OfflineIntegrityReportCompleteness,
    OfflineIntegrityReportDestination, OfflineIntegrityReportWireDenial,
};

use crate::support::{clean_store, StoreFixture};

mod hostile_identity;

#[test]
fn entry_bound_preserves_indeterminate_addressed_root_and_exact_work() {
    let fixture = clean_store("entry-bound");
    let report = observe_store(&bounded_request(&fixture, limits(4, 16 * 1024, 8, 0))).unwrap();
    assert_eq!(
        report.completeness(),
        OfflineIntegrityReportCompleteness::BoundExhausted
    );
    let root = report
        .artifacts()
        .iter()
        .find(|artifact| artifact.family() == PhysicalArtifactFamily::RootManifest)
        .unwrap();
    assert_eq!(
        root.outcome(),
        &OfflineIntegrityOutcome::Indeterminate(
            OfflineIndeterminatePhysicalReason::EntryBoundExceeded
        )
    );
    let counters = report.counters();
    assert_eq!(counters.entries_visited(), 4);
    assert_eq!(counters.bytes_read(), 286);
    assert_eq!(counters.files_opened(), 11);
    assert_eq!(counters.maximum_depth_reached(), 3);
    assert_eq!(counters.checksum_calculations(), 3);
    assert_eq!(counters.namespace_identity_payload_decoder_entries(), 1);
    assert_eq!(counters.selector_payload_decoder_entries(), 2);
    assert_eq!(counters.root_manifest_payload_decoder_entries(), 0);
    assert_eq!(counters.missing_artifacts(), 0);
    assert_eq!(counters.exhausted_bounds(), 1);
}

#[test]
fn partial_directory_prefix_is_reported_without_loss() {
    let fixture = clean_store("partial-prefix");
    std::fs::write(
        fixture
            .records
            .join("root-current-0000000000000002.candidate"),
        b"candidate",
    )
    .unwrap();
    std::fs::remove_file(fixture.roots.join("root-0000000000000001.manifest")).unwrap();
    for generation in [2_u64, 3] {
        std::fs::write(
            fixture
                .roots
                .join(format!("root-{generation:016x}.manifest")),
            crate::support::root_manifest_bytes(),
        )
        .unwrap();
    }
    let report = observe_store(&bounded_request(&fixture, limits(6, 16 * 1024, 8, 0))).unwrap();
    let candidate = report
        .artifacts()
        .iter()
        .find(|artifact| {
            artifact
                .relative_path()
                .ends_with("0000000000000002.candidate")
        })
        .expect("admitted candidate prefix must remain visible");
    assert!(matches!(
        candidate.outcome(),
        OfflineIntegrityOutcome::Damaged(_)
    ));
    assert_eq!(
        report
            .artifacts()
            .iter()
            .find(|artifact| artifact.relative_path().ends_with("root-current.selector"))
            .unwrap()
            .outcome(),
        &OfflineIntegrityOutcome::Intact
    );
    assert_eq!(
        report
            .artifacts()
            .iter()
            .find(|artifact| artifact.relative_path().ends_with("root-previous.selector"))
            .unwrap()
            .outcome(),
        &OfflineIntegrityOutcome::Intact
    );
    let addressed = report
        .artifacts()
        .iter()
        .find(|artifact| {
            artifact.relative_path() == "families/records/roots/root-0000000000000001.manifest"
        })
        .unwrap();
    assert_eq!(
        addressed.outcome(),
        &OfflineIntegrityOutcome::Indeterminate(
            OfflineIndeterminatePhysicalReason::EntryBoundExceeded
        )
    );
    let admitted_root_prefix: Vec<_> = report
        .artifacts()
        .iter()
        .filter(|artifact| {
            artifact
                .relative_path()
                .ends_with("0000000000000002.manifest")
                || artifact
                    .relative_path()
                    .ends_with("0000000000000003.manifest")
        })
        .collect();
    assert_eq!(admitted_root_prefix.len(), 1);
    assert_eq!(
        admitted_root_prefix[0].outcome(),
        &OfflineIntegrityOutcome::Unknown(
            worth_store_offline_integrity_observer::OfflineUnknownPhysicalReason::RootNotAddressed
        )
    );
    assert_eq!(
        report.completeness(),
        OfflineIntegrityReportCompleteness::BoundExhausted
    );
    assert_eq!(report.counters().entries_visited(), 6);
    assert_eq!(report.counters().bytes_read(), 663);
    assert_eq!(report.counters().files_opened(), 20);
    assert_eq!(report.counters().selector_payload_decoder_entries(), 2);
    assert_eq!(report.counters().root_manifest_payload_decoder_entries(), 1);
}

#[test]
fn byte_and_depth_bounds_stop_before_open_or_decode() {
    let byte_fixture = clean_store("byte-bound");
    let byte_report =
        observe_store(&bounded_request(&byte_fixture, limits(100, 100, 8, 0))).unwrap();
    assert_eq!(
        byte_report.completeness(),
        OfflineIntegrityReportCompleteness::BoundExhausted
    );
    assert!(byte_report.artifacts().iter().any(|artifact| {
        artifact.family() == PhysicalArtifactFamily::NamespaceIdentity
            && artifact.outcome() == &OfflineIntegrityOutcome::Intact
    }));
    assert!(byte_report
        .artifacts()
        .iter()
        .filter(|artifact| { artifact.family() != PhysicalArtifactFamily::NamespaceIdentity })
        .all(|artifact| {
            artifact.outcome()
                == &OfflineIntegrityOutcome::Indeterminate(
                    OfflineIndeterminatePhysicalReason::ByteBoundExceeded,
                )
        }));
    let byte_counters = byte_report.counters();
    assert_eq!(byte_counters.entries_visited(), 5);
    assert_eq!(byte_counters.bytes_read(), 72);
    assert_eq!(byte_counters.files_opened(), 3);
    assert_eq!(byte_counters.checksum_calculations(), 1);
    assert_eq!(byte_counters.exhausted_bounds(), 1);

    let depth_fixture = clean_store("depth-bound");
    let depth_report = observe_store(&bounded_request(
        &depth_fixture,
        limits(100, 16 * 1024, 3, 0),
    ))
    .unwrap();
    let root = depth_report
        .artifacts()
        .iter()
        .find(|artifact| artifact.family() == PhysicalArtifactFamily::RootManifest)
        .unwrap();
    assert_eq!(
        root.outcome(),
        &OfflineIntegrityOutcome::Indeterminate(
            OfflineIndeterminatePhysicalReason::DepthBoundExceeded
        )
    );
    let depth_counters = depth_report.counters();
    assert_eq!(depth_counters.entries_visited(), 5);
    assert_eq!(depth_counters.bytes_read(), 286);
    assert_eq!(depth_counters.files_opened(), 11);
    assert_eq!(depth_counters.maximum_depth_reached(), 4);
    assert_eq!(depth_counters.root_manifest_payload_decoder_entries(), 0);
    assert_eq!(depth_counters.exhausted_bounds(), 1);
}

#[test]
fn open_file_bound_is_typed_before_deepest_root_acquisition() {
    let fixture = clean_store("open-file-bound");
    let limits =
        OfflineIntegrityObservationLimits::new(100, 16 * 1024, 4, 8, 0, 5_000, 64 * 1024).unwrap();
    let report = observe_store(&bounded_request(&fixture, limits)).unwrap();
    let root = report
        .artifacts()
        .iter()
        .find(|artifact| artifact.family() == PhysicalArtifactFamily::RootManifest)
        .unwrap();
    assert_eq!(
        root.outcome(),
        &OfflineIntegrityOutcome::Indeterminate(
            OfflineIndeterminatePhysicalReason::OpenFileBoundExceeded
        )
    );
    assert_eq!(
        report.completeness(),
        OfflineIntegrityReportCompleteness::BoundExhausted
    );
    let counters = report.counters();
    assert_eq!(counters.bytes_read(), 286);
    assert_eq!(counters.files_opened(), 15);
    assert_eq!(counters.open_file_high_water(), 4);
    assert_eq!(counters.root_manifest_payload_decoder_entries(), 0);
    assert_eq!(counters.exhausted_bounds(), 1);
}

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
fn report_bound_refuses_emission_without_creating_output() {
    let fixture = clean_store("report-bound");
    let limits = OfflineIntegrityObservationLimits::new(100, 16 * 1024, 5, 8, 0, 5_000, 1).unwrap();
    let denial = observe_store(&bounded_request(&fixture, limits)).unwrap_err();
    assert!(matches!(
        denial,
        OfflineIntegrityObservationDenial::ReportWire(
            OfflineIntegrityReportWireDenial::ReportSizeExceeded { maximum: 1, .. }
        )
    ));
    assert!(!fixture.report.exists());
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

fn limits(
    entries: u64,
    bytes: u64,
    depth: u32,
    symlinks: u64,
) -> OfflineIntegrityObservationLimits {
    OfflineIntegrityObservationLimits::new(entries, bytes, 5, depth, symlinks, 5_000, 64 * 1024)
        .unwrap()
}

fn bounded_request(
    fixture: &StoreFixture,
    limits: OfflineIntegrityObservationLimits,
) -> OfflineIntegrityObservationRequest {
    OfflineIntegrityObservationRequest::new(
        fixture.store.clone(),
        limits,
        OfflineIntegrityReportDestination::file(fixture.report.clone()).unwrap(),
        OfflineIntegrityProtocolContext::new(
            "fixture-observer",
            "process-1",
            "run-1",
            "scenario-1",
        )
        .unwrap(),
    )
    .unwrap()
}

fn store_snapshot(root: &Path) -> Vec<(String, Vec<u8>)> {
    fn visit(root: &Path, directory: &Path, snapshot: &mut Vec<(String, Vec<u8>)>) {
        let mut entries: Vec<_> = std::fs::read_dir(directory)
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .collect();
        entries.sort();
        for path in entries {
            if path.is_dir() {
                visit(root, &path, snapshot);
            } else {
                snapshot.push((
                    path.strip_prefix(root)
                        .unwrap()
                        .to_string_lossy()
                        .replace('\\', "/"),
                    std::fs::read(path).unwrap(),
                ));
            }
        }
    }
    let mut snapshot = Vec::new();
    visit(root, root, &mut snapshot);
    snapshot
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
