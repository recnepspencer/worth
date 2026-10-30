//! Every named Store recovery stage must be reached and killed. Most cases
//! replay a commit killed after WAL durability. Cleanup cases kill a published
//! checkpoint before WAL reclamation, with small certification-only segments
//! and a bounded real redo prefix. Three recoveries, continuation and fresh
//! identity allocation must then agree with the same memory-only reference.

#[allow(unused_imports)]
use crate::recovery_interruption::*;

#[test]
fn a_recovery_killed_at_each_of_its_stages_recovers_the_same_home_three_times() {
    let script = crate::process_kill::script();
    let request = crate::workload::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&request);
    for stage in STAGES {
        assert_eq!(STAGES[asserted_stage_index(stage)], stage);
        if matches!(
            stage,
            worth_store::physical_runtime::PhysicalRecoveryYieldpointStage::CleanupFreshnessRead
                | worth_store::physical_runtime::PhysicalRecoveryYieldpointStage::CleanupRemoval
        ) {
            // A checkpoint published before the serving runtime reclaims WAL
            // leaves real whole covered segments for recovery to remove.
            let bytes = std::num::NonZeroU64::new(512 << 10).unwrap();
            worth_store_relational::RelationalStoreCertificationProfile::with_wal_segment_bytes(
                bytes,
            )
            .run(|| prove_stage(stage, &script, &execution, Some(bytes)));
        } else {
            prove_stage(stage, &script, &execution, None);
        }
    }
    eprintln!("recovery killed at all {} required stages", STAGES.len());
}

#[test]
fn recovery_interruption_child() {
    let Some(label) = std::env::var_os(STAGE_ENV) else {
        return;
    };
    let run = || recover_child_at(label);
    match crate::process_kill::child_wal_profile() {
        Some(profile) => profile.run(run),
        None => run(),
    }
}
