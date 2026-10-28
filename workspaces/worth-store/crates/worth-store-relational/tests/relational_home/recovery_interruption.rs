//! Every named Store recovery stage must be reached and killed. Most cases
//! replay a commit killed after WAL durability. Cleanup cases kill a published
//! checkpoint before WAL reclamation, with small certification-only segments
//! and a bounded real redo prefix. Three recoveries, continuation and fresh
//! identity allocation must then agree with the same memory-only reference.

use std::fs;
use std::{ffi::OsString, num::NonZeroU64, path::PathBuf};

use worth_execution::ExecutionRequest;
use worth_store::physical_runtime::certification::CertificationPhysicalMutationCheckpoint as Seam;
use worth_store::physical_runtime::certification::{MediaFaultDirective, MediaOperationRole};
use worth_store::physical_runtime::production::PhysicalCheckpointStep;
use worth_store::physical_runtime::{
    PhysicalRecoveryProcessYieldpoint, PhysicalRecoveryYieldpointStage as Stage,
};
use worth_store_relational::{
    RelationalStoreCertificationProfile, RelationalStoreHome, RelationalStoreHomeRequest,
};

use super::crash_oracle::{assert_continues, assert_recovered, InFlight};
use super::durable_run::{assert_holds, open, recorded_schema};
use super::killed_child::{self, ChildEnd};
use super::process_kill::{self, Stop, SEED};
use super::schema_transition;
use super::workload;

const STAGE_ENV: &str = "WORTH_RELATIONAL_HOME_RECOVERY_STAGE";
const LOCATION_ENV: &str = "WORTH_RELATIONAL_HOME_RECOVERY_LOCATION";
const MARKERS_ENV: &str = "WORTH_RELATIONAL_HOME_RECOVERY_MARKERS";
const RECOVERY_CHILD: &str = "recovery_interruption::recovery_interruption_child";

const STAGES: [Stage; 12] = [
    Stage::StagingMaterialization,
    Stage::StagingSynchronization,
    Stage::CandidateMaterialization,
    Stage::CandidateSynchronization,
    Stage::RootProtocolReplacement,
    Stage::RecordNamespaceSynchronization,
    Stage::FreshReopenCurrentSelector,
    Stage::FreshReopenRootManifest,
    Stage::FreshReopenExactBinding,
    Stage::CleanupFreshnessRead,
    Stage::CleanupRemoval,
    Stage::ObligationDischarge,
];

// No wildcard: a new Store stage must extend the crash oracle's roster.
fn asserted_stage_index(stage: Stage) -> usize {
    match stage {
        Stage::StagingMaterialization => 0,
        Stage::StagingSynchronization => 1,
        Stage::CandidateMaterialization => 2,
        Stage::CandidateSynchronization => 3,
        Stage::RootProtocolReplacement => 4,
        Stage::RecordNamespaceSynchronization => 5,
        Stage::FreshReopenCurrentSelector => 6,
        Stage::FreshReopenRootManifest => 7,
        Stage::FreshReopenExactBinding => 8,
        Stage::CleanupFreshnessRead => 9,
        Stage::CleanupRemoval => 10,
        Stage::ObligationDischarge => 11,
    }
}

#[test]
fn a_recovery_killed_at_each_of_its_stages_recovers_the_same_home_three_times() {
    let script = process_kill::script();
    let request = workload::serial_request();
    let execution = ExecutionRequest::serial(&request);
    for stage in STAGES {
        assert_eq!(STAGES[asserted_stage_index(stage)], stage);
        if matches!(stage, Stage::CleanupFreshnessRead | Stage::CleanupRemoval) {
            // A checkpoint published before the serving runtime reclaims WAL
            // leaves real whole covered segments for recovery to remove.
            let bytes = NonZeroU64::new(512 << 10).unwrap();
            RelationalStoreCertificationProfile::with_wal_segment_bytes(bytes)
                .run(|| prove_stage(stage, &script, &execution, Some(bytes)));
        } else {
            prove_stage(stage, &script, &execution, None);
        }
    }
    eprintln!("recovery killed at all {} required stages", STAGES.len());
}

fn prove_stage(
    stage: Stage,
    script: &workload::Script,
    execution: &ExecutionRequest<'_, '_>,
    wal_segment_bytes: Option<NonZeroU64>,
) {
    let case = format!("seed {SEED:#x} recovery killed at {}", stage.label());
    let (stop, in_flight) = if wal_segment_bytes.is_some() {
        (
            Stop::CheckpointAcknowledged(PhysicalCheckpointStep::NamespaceSynchronization),
            InFlight::Nothing,
        )
    } else {
        (Stop::Commit(Seam::AfterWalDurability), InFlight::Durable)
    };
    let killed = process_kill::launch_with_wal_segment_bytes(stop, wal_segment_bytes);
    if stage == Stage::ObligationDischarge {
        leave_indeterminate_replacement(&killed.location, execution);
    }
    let markers = tempfile::tempdir().unwrap();
    let label = PathBuf::from(stage.label());
    let bytes = wal_segment_bytes.map(|bytes| PathBuf::from(bytes.get().to_string()));
    let mut environment: Vec<(&str, &std::path::Path)> = vec![
        (STAGE_ENV, &label),
        (LOCATION_ENV, &killed.location),
        (MARKERS_ENV, markers.path()),
    ];
    if let Some(bytes) = &bytes {
        environment.push((process_kill::WAL_SEGMENT_BYTES_ENV, bytes));
    }
    let end = killed_child::run_and_kill(
        RECOVERY_CHILD,
        &environment,
        &markers.path().join("reached"),
        &markers.path().join("finished"),
    );
    assert_eq!(
        end,
        ChildEnd::KilledAtPark,
        "required recovery stage was not killed: {}",
        stage.label()
    );
    eprintln!("killed recovery stage: {}", stage.label());

    let home = open(&killed.location, execution)
        .unwrap_or_else(|refusal| panic!("{case}: recovery refused: {refusal:?}"));
    let commits = assert_recovered(&home, script, &killed.acknowledged, &in_flight, &case);
    let recovered = workload::artifacts_of(home.runtime());
    home.close();
    for attempt in ["second", "third"] {
        let again = open(&killed.location, execution)
            .unwrap_or_else(|refusal| panic!("{case}: {attempt} recovery: {refusal:?}"));
        assert_holds(&again, script, &recovered);
        if attempt == "third" {
            assert_continues(&killed.location, again, script, commits, execution, &case);
        } else {
            again.close();
        }
    }
}

/// A settled WAL seam leaves no physical-work obligation. Lose the answer
/// to a real root replacement so recovery must adjudicate and discharge one.
/// Checkpointing changes no logical history, even when its selection is unclear.
fn leave_indeterminate_replacement(
    location: &std::path::Path,
    execution: &ExecutionRequest<'_, '_>,
) {
    let fault = (
        MediaOperationRole::AtomicReplace,
        NonZeroU64::new(1).unwrap(),
        MediaFaultDirective::IndeterminateAfterEffect,
    );
    let (home, activations) = super::media_faults::open_with_faults(
        location,
        schema_transition::TARGET,
        &[fault],
        execution,
    )
    .unwrap();
    super::media_faults::arm(&activations);
    assert!(home.checkpoint(execution).is_err());
    assert!(activations[0].is_consumed());
    home.certification_abort();
}

#[test]
fn recovery_interruption_child() {
    let Some(label) = std::env::var_os(STAGE_ENV) else {
        return;
    };
    let run = || recover_child_at(label);
    match process_kill::child_wal_profile() {
        Some(profile) => profile.run(run),
        None => run(),
    }
}

fn recover_child_at(label: OsString) {
    let stage = Stage::from_label(label.to_str().unwrap()).expect("a recovery stage label");
    let location = PathBuf::from(std::env::var_os(LOCATION_ENV).unwrap());
    let markers = PathBuf::from(std::env::var_os(MARKERS_ENV).unwrap());
    let yieldpoint = PhysicalRecoveryProcessYieldpoint::new(
        stage,
        markers.join("reached"),
        markers.join("release"),
        markers.join("cancel"),
    );
    let request = workload::serial_request();
    let execution = ExecutionRequest::serial(&request);
    let opened = RelationalStoreHome::certification_open_with_recovery_yieldpoint(
        RelationalStoreHomeRequest::new(
            &location,
            workload::runtime_builder(schema_transition::SOURCE),
        ),
        yieldpoint,
        &execution,
    );
    // The commit fixture includes a durable schema transition, so a completed
    // recovery serves its target. The checkpoint fixture still serves SOURCE.
    let home = match opened {
        Err(refusal) if recorded_schema(&refusal) == Some(schema_transition::TARGET) => {
            open(&location, &execution)
        }
        opened => opened,
    };
    home.expect("a recovery that never parks completes").close();
    fs::write(markers.join("finished"), b"1").unwrap();
}
