pub(crate) use crate::support::fixtures::{
    clean_store, current_selector_bytes, namespace_identity_bytes, previous_selector_bytes,
    root_manifest_bytes, StoreFixture,
};
pub(crate) use crate::support::json::parse_json;
pub(crate) use crate::support::temporary_root::TemporaryRoot;

/// Wall-clock budget for fixture observations under parallel suite load.
pub(crate) const FIXTURE_ELAPSED_MILLISECONDS: u64 = 120_000;
/// The same budget as a command-line argument.
pub(crate) const FIXTURE_ELAPSED_ARGUMENT: &str = "120000";
/// Windows charges one transient identity handle above the held read path.
pub(crate) const FIXTURE_OPEN_FILE_HANDLES: u32 = if cfg!(windows) { 6 } else { 5 };
pub(crate) const FIXTURE_OPEN_FILE_ARGUMENT: &str = if cfg!(windows) { "6" } else { "5" };

pub(crate) use std::path::PathBuf;

pub(crate) use worth_store_offline_integrity_observer::{
    OfflineIntegrityObservationLimits, OfflineIntegrityObservationRequest,
    OfflineIntegrityProtocolContext, OfflineIntegrityReportDestination,
};

pub(crate) fn request(fixture: &StoreFixture) -> OfflineIntegrityObservationRequest {
    worth_store_offline_integrity_observer::OfflineIntegrityObservationRequest::new(
        fixture.store.clone(),
        worth_store_offline_integrity_observer::OfflineIntegrityObservationLimits::new(
            100,
            16 * 1024,
            FIXTURE_OPEN_FILE_HANDLES,
            8,
            0,
            FIXTURE_ELAPSED_MILLISECONDS,
            64 * 1024,
        )
        .unwrap(),
        worth_store_offline_integrity_observer::OfflineIntegrityReportDestination::file(
            fixture.report.clone(),
        )
        .unwrap(),
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

pub(crate) fn artifact_path(fixture: &StoreFixture, target: Target) -> PathBuf {
    match target {
        crate::store_offline_integrity_observer_support::Target::Current => {
            fixture.records.join("root-current.selector")
        }
        crate::store_offline_integrity_observer_support::Target::Previous => {
            fixture.records.join("root-previous.selector")
        }
        crate::store_offline_integrity_observer_support::Target::Root => {
            fixture.roots.join("root-0000000000000001.manifest")
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Target {
    Current,
    Previous,
    Root,
}
