pub(crate) use std::fs;

pub(crate) use worth_foundational::PhysicalIntegrityPosture;
pub(crate) use worth_store_offline_integrity_observer::{
    observe_store, OfflineArtifactObservation, OfflineIntegrityOutcome, OfflinePhysicalDamageCause,
    OfflinePhysicalFormatField, OfflineUnknownPhysicalReason,
};

pub(crate) use crate::{
    store_offline_integrity_observer_support::artifact_path,
    store_offline_integrity_observer_support::request,
    store_offline_integrity_observer_support::Target, support::fixtures::clean_store,
    support::fixtures::current_selector_bytes,
};

pub(crate) fn target_observation(
    report: &worth_store_offline_integrity_observer::OfflineIntegrityReport,
    target: Target,
) -> &OfflineArtifactObservation {
    let path = match target {
        crate::store_offline_integrity_observer_support::Target::Current => {
            "families/records/root-current.selector"
        }
        crate::store_offline_integrity_observer_support::Target::Previous => {
            "families/records/root-previous.selector"
        }
        crate::store_offline_integrity_observer_support::Target::Root => {
            "families/records/roots/root-0000000000000001.manifest"
        }
    };
    report
        .artifacts()
        .iter()
        .find(|artifact| artifact.relative_path() == path)
        .unwrap()
}
