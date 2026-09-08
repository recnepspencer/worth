#[cfg(test)]
mod artifact_editor;
#[cfg(test)]
mod clean_artifact_manifest;
#[cfg(test)]
mod corruption_operator;
#[cfg(test)]
mod counters;
#[cfg(test)]
mod editor_result_audit;
mod external_report_paths;
#[cfg(test)]
mod external_report_paths_tests;
#[cfg(test)]
mod frame_checksum;
#[cfg(test)]
mod parent_oracle;
#[cfg(test)]
mod producer_fixture;
mod root_artifact_role;
#[cfg(test)]
mod scenario;
#[cfg(test)]
mod wire;

#[cfg(test)]
mod offline_process;
#[cfg(test)]
mod oracle_expectation_assertions;
#[cfg(test)]
mod process_courtroom;
#[cfg(test)]
mod process_courtroom_assertions;
#[cfg(test)]
mod process_identity_substitution;
#[cfg(test)]
mod process_integrity_vocabulary;
#[cfg(test)]
mod process_manifest;
#[cfg(test)]
mod process_poison;
#[cfg(test)]
mod process_protocol;
#[cfg(test)]
mod process_recovery_observation;
#[cfg(test)]
mod process_subject;
#[cfg(test)]
mod production_store;
#[cfg(test)]
mod recovery_adapter;
#[cfg(test)]
mod recovery_request;
#[cfg(test)]
mod test_world;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod wire_tests;

#[cfg(test)]
pub(crate) use artifact_editor::apply_declared_corruption;
#[cfg(test)]
pub(crate) use clean_artifact_manifest::{
    CleanRootArtifactManifest, CleanRootArtifactRecord, RootArtifactIdentity,
    RootArtifactManifestDenial,
};
#[cfg(test)]
pub(crate) use corruption_operator::{DeclaredRootCorruption, RootCorruptionCode};
#[cfg(test)]
pub(crate) use counters::RootLocalizationCounters;
#[cfg(test)]
pub(crate) use editor_result_audit::{EditorAuditDenial, EditorResultAudit};
#[cfg(test)]
pub(crate) use parent_oracle::{
    derive_parent_expectation, ExpectedMinimumBlastRadius, ExpectedRootCause,
    ExpectedRootLocalization, ExpectedRootPosture,
};
pub(crate) use root_artifact_role::RootArtifactRole;
#[cfg(test)]
pub(crate) use scenario::{FreshRootArtifactRow, FreshRootArtifactRowDenial, RootSliceScenario};
#[cfg(test)]
pub(crate) use wire::RootWireDenial;
#[cfg(test)]
pub(crate) use wire::{RootWireIdentity, RootWireRole};

#[cfg(test)]
pub(crate) use external_report_paths::ExternalReportPathDenial;
pub(crate) use external_report_paths::ExternalReportPaths;
#[cfg(test)]
pub(crate) use process_manifest::ClosedStoreProcessManifest;
#[cfg(test)]
pub(crate) use process_poison::{
    apply_process_poison, DeclaredProcessPoison, ProcessEditorAudit, ProcessRootCase,
};

#[cfg(test)]
const OBSERVER_EXECUTABLE_ENV: &str = "WORTH_C9_OBSERVER_EXECUTABLE";

#[cfg(test)]
#[test]
#[ignore = "requires the Cargo-built observer; run the C.9 courtroom script"]
fn c9_root_protocol_process_courtroom() {
    let executable = std::env::var_os(OBSERVER_EXECUTABLE_ENV).unwrap_or_else(|| {
        panic!(
            "{OBSERVER_EXECUTABLE_ENV} is required; run scripts/ci/run_worth_store_c9_root_process_courtroom.py"
        )
    });
    process_courtroom::run(std::path::Path::new(&executable));
}

#[cfg(test)]
#[test]
fn c9_root_process_subject() {
    process_subject::run();
}
