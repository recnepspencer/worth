//! Every named Store recovery stage must be reached and killed. Most cases
//! replay a commit killed after WAL durability. Cleanup cases kill a published
//! checkpoint before WAL reclamation, with small certification-only segments
//! and a bounded real redo prefix. Three recoveries, continuation and fresh
//! identity allocation must then agree with the same memory-only reference.

#[allow(unused_imports)] use crate::recovery_interruption::*;
use {std::fs};
use {std::ffi::OsString, std::num::NonZeroU64, std::path::PathBuf};

use {worth_execution::ExecutionRequest};
use {worth_store::physical_runtime::certification::CertificationPhysicalMutationCheckpoint as Seam};
use {worth_store::physical_runtime::certification::MediaFaultDirective, worth_store::physical_runtime::certification::MediaOperationRole};
use {worth_store::physical_runtime::production::PhysicalCheckpointStep};
use {worth_store::physical_runtime::PhysicalRecoveryProcessYieldpoint, worth_store::physical_runtime::PhysicalRecoveryYieldpointStage as Stage};
use {worth_store_relational::RelationalStoreCertificationProfile, worth_store_relational::RelationalStoreHome, worth_store_relational::RelationalStoreHomeRequest};

use {crate::crash_oracle::assert_continues, crate::crash_oracle::assert_recovered, crate::crash_oracle::InFlight};
use {crate::durable_run::assert_holds, crate::durable_run::open, crate::durable_run::recorded_schema};
use {crate::killed_child, crate::killed_child::ChildEnd};
use {crate::process_kill, crate::process_kill::Stop, crate::process_kill::SEED};
use {crate::schema_transition};
use {crate::workload};

#[test]
fn a_recovery_killed_at_each_of_its_stages_recovers_the_same_home_three_times() {
    let script = crate::process_kill::script();
    let request = crate::workload::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&request);
    for stage in STAGES {
        assert_eq!(STAGES[asserted_stage_index(stage)], stage);
        if matches!(stage, worth_store::physical_runtime::PhysicalRecoveryYieldpointStage::CleanupFreshnessRead | worth_store::physical_runtime::PhysicalRecoveryYieldpointStage::CleanupRemoval) {
            // A checkpoint published before the serving runtime reclaims WAL
            // leaves real whole covered segments for recovery to remove.
            let bytes = std::num::NonZeroU64::new(512 << 10).unwrap();
            worth_store_relational::RelationalStoreCertificationProfile::with_wal_segment_bytes(bytes)
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
