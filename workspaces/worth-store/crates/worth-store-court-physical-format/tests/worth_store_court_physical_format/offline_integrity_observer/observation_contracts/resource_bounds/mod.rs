#[allow(unused_imports)]
use crate::resource_bounds::*;

#[test]
fn entry_bound_preserves_indeterminate_addressed_root_and_exact_work() {
    let fixture = clean_store("entry-bound");
    let report = observe_store(&bounded_request(&fixture, limits(4, 16 * 1024, 8, 0))).unwrap();
    assert_eq!(
        report.completeness(),
        worth_store_offline_integrity_observer::OfflineIntegrityReportCompleteness::BoundExhausted
    );
    let root = report
        .artifacts()
        .iter()
        .find(|artifact| {
            artifact.family() == worth_foundational::PhysicalArtifactFamily::RootManifest
        })
        .unwrap();
    assert_eq!(
        root.outcome(),
        &worth_store_offline_integrity_observer::OfflineIntegrityOutcome::Indeterminate(
            worth_store_offline_integrity_observer::OfflineIndeterminatePhysicalReason::EntryBoundExceeded
        )
    );
    let counters = report.counters();
    assert_eq!(counters.entries_visited(), 4);
    assert_eq!(counters.bytes_read(), 286);
    assert_eq!(counters.files_opened(), if cfg!(windows) { 20 } else { 11 });
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
            crate::support::fixtures::root_manifest_bytes(),
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
        worth_store_offline_integrity_observer::OfflineIntegrityOutcome::Damaged(_)
    ));
    assert_eq!(
        report
            .artifacts()
            .iter()
            .find(|artifact| artifact.relative_path().ends_with("root-current.selector"))
            .unwrap()
            .outcome(),
        &worth_store_offline_integrity_observer::OfflineIntegrityOutcome::Intact
    );
    assert_eq!(
        report
            .artifacts()
            .iter()
            .find(|artifact| artifact.relative_path().ends_with("root-previous.selector"))
            .unwrap()
            .outcome(),
        &worth_store_offline_integrity_observer::OfflineIntegrityOutcome::Intact
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
        &worth_store_offline_integrity_observer::OfflineIntegrityOutcome::Indeterminate(
            worth_store_offline_integrity_observer::OfflineIndeterminatePhysicalReason::EntryBoundExceeded
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
        &worth_store_offline_integrity_observer::OfflineIntegrityOutcome::Unknown(
            worth_store_offline_integrity_observer::OfflineUnknownPhysicalReason::RootNotAddressed
        )
    );
    assert_eq!(
        report.completeness(),
        worth_store_offline_integrity_observer::OfflineIntegrityReportCompleteness::BoundExhausted
    );
    assert_eq!(report.counters().entries_visited(), 6);
    assert_eq!(report.counters().bytes_read(), 679);
    assert_eq!(
        report.counters().files_opened(),
        if cfg!(windows) { 35 } else { 20 }
    );
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
        worth_store_offline_integrity_observer::OfflineIntegrityReportCompleteness::BoundExhausted
    );
    assert!(byte_report.artifacts().iter().any(|artifact| {
        artifact.family() == worth_foundational::PhysicalArtifactFamily::NamespaceIdentity
            && artifact.outcome()
                == &worth_store_offline_integrity_observer::OfflineIntegrityOutcome::Intact
    }));
    assert!(byte_report
        .artifacts()
        .iter()
        .filter(|artifact| { artifact.family() != worth_foundational::PhysicalArtifactFamily::NamespaceIdentity })
        .all(|artifact| {
            artifact.outcome()
                == &worth_store_offline_integrity_observer::OfflineIntegrityOutcome::Indeterminate(
                    worth_store_offline_integrity_observer::OfflineIndeterminatePhysicalReason::ByteBoundExceeded,
                )
        }));
    let byte_counters = byte_report.counters();
    assert_eq!(byte_counters.entries_visited(), 5);
    assert_eq!(byte_counters.bytes_read(), 72);
    assert_eq!(
        byte_counters.files_opened(),
        if cfg!(windows) { 6 } else { 3 }
    );
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
        .find(|artifact| {
            artifact.family() == worth_foundational::PhysicalArtifactFamily::RootManifest
        })
        .unwrap();
    assert_eq!(
        root.outcome(),
        &worth_store_offline_integrity_observer::OfflineIntegrityOutcome::Indeterminate(
            worth_store_offline_integrity_observer::OfflineIndeterminatePhysicalReason::DepthBoundExceeded
        )
    );
    let depth_counters = depth_report.counters();
    assert_eq!(depth_counters.entries_visited(), 5);
    assert_eq!(depth_counters.bytes_read(), 286);
    assert_eq!(
        depth_counters.files_opened(),
        if cfg!(windows) { 20 } else { 11 }
    );
    assert_eq!(depth_counters.maximum_depth_reached(), 4);
    assert_eq!(depth_counters.root_manifest_payload_decoder_entries(), 0);
    assert_eq!(depth_counters.exhausted_bounds(), 1);
}

#[test]
fn open_file_bound_is_typed_before_deepest_root_acquisition() {
    let fixture = clean_store("open-file-bound");
    let limits = worth_store_offline_integrity_observer::OfflineIntegrityObservationLimits::new(
        100,
        16 * 1024,
        if cfg!(windows) { 5 } else { 4 },
        8,
        0,
        crate::store_offline_integrity_observer_support::FIXTURE_ELAPSED_MILLISECONDS,
        64 * 1024,
    )
    .unwrap();
    let report = observe_store(&bounded_request(&fixture, limits)).unwrap();
    let root = report
        .artifacts()
        .iter()
        .find(|artifact| {
            artifact.family() == worth_foundational::PhysicalArtifactFamily::RootManifest
        })
        .unwrap();
    assert_eq!(
        root.outcome(),
        &worth_store_offline_integrity_observer::OfflineIntegrityOutcome::Indeterminate(
            worth_store_offline_integrity_observer::OfflineIndeterminatePhysicalReason::OpenFileBoundExceeded
        )
    );
    assert_eq!(
        report.completeness(),
        worth_store_offline_integrity_observer::OfflineIntegrityReportCompleteness::BoundExhausted
    );
    let counters = report.counters();
    assert_eq!(counters.bytes_read(), 286);
    assert_eq!(counters.files_opened(), if cfg!(windows) { 25 } else { 15 });
    assert_eq!(
        counters.open_file_high_water(),
        if cfg!(windows) { 5 } else { 4 }
    );
    assert_eq!(counters.root_manifest_payload_decoder_entries(), 0);
    assert_eq!(counters.exhausted_bounds(), 1);
}

#[test]
fn report_bound_refuses_emission_without_creating_output() {
    let fixture = clean_store("report-bound");
    let limits = worth_store_offline_integrity_observer::OfflineIntegrityObservationLimits::new(
        100,
        16 * 1024,
        crate::store_offline_integrity_observer_support::FIXTURE_OPEN_FILE_HANDLES,
        8,
        0,
        crate::store_offline_integrity_observer_support::FIXTURE_ELAPSED_MILLISECONDS,
        1,
    )
    .unwrap();
    let denial = observe_store(&bounded_request(&fixture, limits)).unwrap_err();
    assert!(matches!(
        denial,
        worth_store_offline_integrity_observer::OfflineIntegrityObservationDenial::ReportWire(
            worth_store_offline_integrity_observer::OfflineIntegrityReportWireDenial::ReportSizeExceeded { maximum: 1, .. }
        )
    ));
    assert!(!fixture.report.exists());
}
#[path = "hostile_identity/mod.rs"]
pub(crate) mod hostile_identity;
#[path = "symlink_refusal/mod.rs"]
pub(crate) mod symlink_refusal;
