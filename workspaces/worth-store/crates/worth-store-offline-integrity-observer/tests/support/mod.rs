mod checksum;
mod fixtures;
mod json;
mod temporary_root;

pub(crate) use checksum::{checksum, refresh_crc32c};
pub(crate) use fixtures::{
    clean_store, current_selector_bytes, namespace_identity_bytes, previous_selector_bytes,
    root_manifest_bytes, StoreFixture,
};
pub(crate) use json::parse_json;
pub(crate) use temporary_root::TemporaryRoot;

use std::path::PathBuf;

use worth_store_offline_integrity_observer::{
    OfflineIntegrityObservationLimits, OfflineIntegrityObservationRequest,
    OfflineIntegrityProtocolContext, OfflineIntegrityReportDestination,
};

pub(crate) fn request(fixture: &StoreFixture) -> OfflineIntegrityObservationRequest {
    OfflineIntegrityObservationRequest::new(
        fixture.store.clone(),
        OfflineIntegrityObservationLimits::new(100, 16 * 1024, 5, 8, 0, 5_000, 64 * 1024).unwrap(),
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

pub(crate) fn artifact_path(fixture: &StoreFixture, target: Target) -> PathBuf {
    match target {
        Target::Current => fixture.records.join("root-current.selector"),
        Target::Previous => fixture.records.join("root-previous.selector"),
        Target::Root => fixture.roots.join("root-0000000000000001.manifest"),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Target {
    Current,
    Previous,
    Root,
}
