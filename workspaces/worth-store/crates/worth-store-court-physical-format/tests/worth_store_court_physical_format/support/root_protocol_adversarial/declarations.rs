pub(crate) use {std::fs};

pub(crate) use {worth_foundational::PhysicalIntegrityPosture};
pub(crate) use {worth_store_offline_integrity_observer::observe_store, worth_store_offline_integrity_observer::OfflineArtifactObservation, worth_store_offline_integrity_observer::OfflineIntegrityOutcome, worth_store_offline_integrity_observer::OfflinePhysicalDamageCause, worth_store_offline_integrity_observer::OfflinePhysicalFormatField, worth_store_offline_integrity_observer::OfflineUnknownPhysicalReason};

pub(crate) use {crate::store_offline_integrity_observer_support::artifact_path, crate::support::fixtures::clean_store, crate::support::fixtures::current_selector_bytes, crate::store_offline_integrity_observer_support::request, crate::store_offline_integrity_observer_support::Target};

pub(crate) fn target_observation(
    report: &worth_store_offline_integrity_observer::OfflineIntegrityReport,
    target: Target,
) -> &OfflineArtifactObservation {
    let path = match target {
        crate::store_offline_integrity_observer_support::Target::Current => "families/records/root-current.selector",
        crate::store_offline_integrity_observer_support::Target::Previous => "families/records/root-previous.selector",
        crate::store_offline_integrity_observer_support::Target::Root => "families/records/roots/root-0000000000000001.manifest",
    };
    report
        .artifacts()
        .iter()
        .find(|artifact| artifact.relative_path() == path)
        .unwrap()
}