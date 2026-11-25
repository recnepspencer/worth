//! The original request can stop replay verification and checkpoint decoding.
use worth_execution::{CancellationSource, ExecutionRequest, SerialMemoryBudget, SerialRequest};
use worth_relational::facade::{
    durability::{RelationalNativeCheckpoint, RelationalRecoveryStop},
    history::BranchId,
    replay::{RelationalReplayRequest, ReplayExecutionMode, ReplayVerificationMode},
    transactions::RelationalExecutionDenialCause as Cause,
};
use worth_testkit_relational::{
    durable_publication::{create_entity, memory_runtime},
    shared_relational_certification_test_execution_policy::serial_request,
};

#[test]
fn verification_replay_returns_control_outside_its_serialized_outcome() {
    let source = memory_runtime();
    let setup = serial_request();
    let execution = ExecutionRequest::serial(&setup);
    for name in ["replay-first", "replay-second", "replay-third"] {
        create_entity(&execution, &source, name);
    }
    let artifacts = source
        .durability()
        .canonical_commit_artifacts(&execution)
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    assert_eq!(artifacts.len(), 3);
    let latest_commit = artifacts.last().unwrap().commit_id();
    drop(artifacts);
    let cancellation = CancellationSource::new();
    cancellation.cancel();
    let host =
        SerialRequest::from_memory(SerialMemoryBudget::new(None), cancellation.token(), None);
    let request = RelationalReplayRequest {
        commit_id: latest_commit,
        branch_id: BranchId("main".into()),
        execution_mode: ReplayExecutionMode::SerialDeterministic,
        verification_mode: ReplayVerificationMode::NormalRecoveryVerification,
    };
    let error = source
        .replay_authority()
        .replay_commit(request, &ExecutionRequest::serial(&host))
        .unwrap_err();
    assert_eq!(error, RelationalRecoveryStop::Request(Cause::Cancelled));
}

#[test]
fn cancelled_checkpoint_restore_consults_before_decoding_corrupt_bytes() {
    let mut target = memory_runtime();
    let before = format!("{target:#?}");
    let cancellation = CancellationSource::new();
    cancellation.cancel();
    let host =
        SerialRequest::from_memory(SerialMemoryBudget::new(None), cancellation.token(), None);
    let corrupt = RelationalNativeCheckpoint::from_untrusted_bytes(vec![0xff]);
    assert_eq!(
        target
            .durability_recovery()
            .restore_native_checkpoint(
                &ExecutionRequest::serial(&host),
                &corrupt,
                worth_testkit_relational::cold_recovery::members_source(&corrupt),
            )
            .unwrap_err(),
        RelationalRecoveryStop::Request(Cause::Cancelled),
    );
    assert_eq!(
        format!("{target:#?}"),
        before,
        "entry stop changed target or diagnostics"
    );
}
