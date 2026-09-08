//! C8 receives a separate exact poisoned world, never a scrub verdict.
use super::{
    artifact_inventory::ArtifactGranule,
    process_execution::{fresh_identity, run_subject},
    process_ingress_observation::ProcessIngressOutcome,
    process_manifest::ProcessTreeSnapshot,
    process_protocol::{executable_sha256, ProcessReportPayload, ProcessSubjectRequest},
    process_recovery_observation::{
        ProcessDamageCause, ProcessIntegrityRejection, ProcessRecoveryObservation,
        ProcessRecoveryPosture,
    },
    RootWireRole,
};
use std::path::Path;
mod root_protocol;

pub(super) fn observe(
    root: &Path,
    reports: &Path,
    label: &str,
    store: [u8; 16],
    target: Option<&ArtifactGranule>,
) -> ProcessRecoveryObservation {
    let snapshot = ProcessTreeSnapshot::observe(root).unwrap();
    let world = tempfile::tempdir_in(root.parent().unwrap()).unwrap();
    let copy = world.path().join("recovery-root");
    snapshot.copy_to(root, &copy);
    let executable = std::env::current_exe().unwrap();
    let scenario = fresh_identity(&format!("{label}-c8-scenario"), reports);
    let run = fresh_identity(&format!("{label}-c8-run"), reports);
    let execution = run_subject(
        &executable,
        reports,
        &format!("{label}-c8"),
        ProcessSubjectRequest::recovery(
            scenario,
            run,
            copy,
            reports.join(format!("{label}-c8.report")),
            store,
        ),
    );
    execution
        .report
        .require(
            RootWireRole::Recovery,
            scenario,
            run,
            store,
            execution.process_id,
            executable_sha256(&executable).unwrap(),
        )
        .unwrap();
    snapshot.require_unchanged(root).unwrap();
    let ProcessReportPayload::Recovered(observed) = execution.report.payload() else {
        panic!("fresh C8 role");
    };
    observed.require_store_identity(store).unwrap();
    if target.is_none() {
        assert_eq!(observed.posture, ProcessRecoveryPosture::Recovered);
        assert!(observed.integrity_counters.admitted > 0);
        assert!(
            observed.integrity_counters.owner_decoder_entries
                + observed.integrity_counters.owner_projection_entries
                > 0
        );
    } else {
        let target = target.unwrap();
        let expected = super::process_integrity_projection::project_integrity_scope(target.scope);
        let matches = observed
            .ingress
            .iter()
            .chain(&observed.wal)
            .filter(|row| {
                row.scope.family == expected.family
                    && row.scope.identity == expected.identity
                    && row.scope.byte_range.offset == expected.byte_range.offset
            })
            .collect::<Vec<_>>();
        for row in &matches {
            let ProcessIngressOutcome::Integrity(ProcessIntegrityRejection::Damaged(damage)) =
                row.outcome
            else {
                panic!("C8 consumed checksum-poisoned artifact without rejecting it: {row:?}");
            };
            // Root-protocol B/S refusals have their own projection below;
            // record/WAL ingress is exercised here by the covered-byte row.
            assert_eq!(damage.cause, ProcessDamageCause::ChecksumMismatch);
            assert_eq!(damage.scope.store_identity, store);
        }
        println!(
            "C9 C8 family={} matching_rejections={} posture={:?} counters={:?}",
            target.family,
            matches.len(),
            observed.posture,
            observed.integrity_counters
        );
    }
    observed.clone()
}

pub(super) fn require_consumption(
    clean: &ProcessRecoveryObservation,
    observed: &ProcessRecoveryObservation,
    target: &ArtifactGranule,
    operator: super::artifact_edit::ArtifactOperator,
) {
    if root_protocol::require(clean, observed, target, operator) {
        return;
    }
    let expected = super::process_integrity_projection::project_integrity_scope(target.scope);
    let matches = |row: &&super::process_ingress_observation::ProcessIngressObservation| {
        row.scope.family == expected.family
            && row.scope.identity == expected.identity
            && row.scope.byte_range.offset == expected.byte_range.offset
    };
    let selected = clean
        .ingress
        .iter()
        .chain(&clean.wal)
        .filter(matches)
        .count();
    let consumed = observed
        .ingress
        .iter()
        .chain(&observed.wal)
        .filter(matches)
        .count();
    if selected > 0 {
        assert!(
            consumed > 0,
            "C8 must expose the rejection for a source it actually consumes: {}",
            target.family
        );
    } else {
        assert_eq!(
            consumed, 0,
            "do not invent recovery consumption for a historical/unselected source"
        );
    }
}
