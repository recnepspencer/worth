pub(crate) use {std::path::Path};

pub(crate) use {worth_foundational::PhysicalArtifactFamily};
pub(crate) use {worth_store_offline_integrity_observer::observe_store, worth_store_offline_integrity_observer::OfflineArtifactDuplicateEvidence, worth_store_offline_integrity_observer::OfflineArtifactFamily, worth_store_offline_integrity_observer::OfflineIndeterminatePhysicalReason, worth_store_offline_integrity_observer::OfflineIntegrityObservationDenial, worth_store_offline_integrity_observer::OfflineIntegrityObservationLimits, worth_store_offline_integrity_observer::OfflineIntegrityObservationRequest, worth_store_offline_integrity_observer::OfflineIntegrityOutcome, worth_store_offline_integrity_observer::OfflineIntegrityProtocolContext, worth_store_offline_integrity_observer::OfflineIntegrityReportCompleteness, worth_store_offline_integrity_observer::OfflineIntegrityReportDestination, worth_store_offline_integrity_observer::OfflineIntegrityReportWireDenial};

pub(crate) use {crate::support::fixtures::clean_store, crate::support::fixtures::StoreFixture};

pub(crate) fn limits(
    entries: u64,
    bytes: u64,
    depth: u32,
    symlinks: u64,
) -> OfflineIntegrityObservationLimits {
    worth_store_offline_integrity_observer::OfflineIntegrityObservationLimits::new(
        entries,
        bytes,
        crate::store_offline_integrity_observer_support::FIXTURE_OPEN_FILE_HANDLES,
        depth,
        symlinks,
        crate::store_offline_integrity_observer_support::FIXTURE_ELAPSED_MILLISECONDS,
        64 * 1024,
    )
    .unwrap()
}

pub(crate) fn bounded_request(
    fixture: &StoreFixture,
    limits: OfflineIntegrityObservationLimits,
) -> OfflineIntegrityObservationRequest {
    worth_store_offline_integrity_observer::OfflineIntegrityObservationRequest::new(
        fixture.store.clone(),
        limits,
        worth_store_offline_integrity_observer::OfflineIntegrityReportDestination::file(fixture.report.clone()).unwrap(),
        worth_store_offline_integrity_observer::OfflineIntegrityProtocolContext::new(
            "fixture-observer",
            "process-1",
            "run-1",
            "scenario-1",
        )
        .unwrap(),
    )
    .unwrap()
}

pub(crate) fn store_snapshot(root: &Path) -> Vec<(String, Vec<u8>)> {
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