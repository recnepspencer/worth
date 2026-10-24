//! Recovery readmits everything a medium hands back and refuses, before any
//! effect, whatever is damaged, missing, repeated or misfiled.

use crate::durability::data::{RecoveryVerificationMode, RelationalRecoveryStop};
use crate::facade::durability::RecoveryFailureClass;
use crate::facade::history::BranchId;
use crate::publication::patch::data::PatchStreamPosition;
use crate::runtime::RelationalRuntime;
use crate::tests::support::*;

fn runtime_with_commits(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    names: &[&str],
) -> RelationalRuntime {
    let runtime = durable_runtime_with_test_schema();
    for name in names {
        let committed = create_entity_outcome(execution, &runtime, name);
        release_test_commit_snapshot(&runtime, &committed);
    }
    runtime
}

/// Plan a fresh runtime's recovery from `source`'s medium; return the class
/// of the refusal and prove the fresh runtime is untouched.
fn refusal_class(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    source: &RelationalRuntime,
) -> RecoveryFailureClass {
    let mut recovering = durable_runtime_with_test_schema();
    let refused = medium_recovery_plan(
        execution,
        source,
        &mut recovering,
        RecoveryVerificationMode::NormalRecoveryVerification,
    )
    .expect_err("damaged durable history is refused before service");
    assert!(recovering
        .history()
        .branch_head(&BranchId("main".to_owned()))
        .is_none());
    match refused {
        RelationalRecoveryStop::Durability(error) => error.class,
        other => panic!("expected a typed durable refusal, got {other:?}"),
    }
}

#[test]
fn an_artifact_whose_frame_is_damaged_is_refused_at_the_read() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);
    let runtime = runtime_with_commits(execution, &["a", "b"]);
    let medium = attached_test_medium(&runtime);
    let last_byte = medium.artifact_bytes(PatchStreamPosition(2)).unwrap().len() - 1;

    medium.flip_artifact_byte(PatchStreamPosition(2), last_byte);

    assert!(
        medium.recovery_source().is_err(),
        "a payload that no longer matches its digest never becomes an artifact"
    );
}

#[test]
fn an_artifact_that_does_not_readmit_refuses_as_corrupt() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);
    let runtime = runtime_with_commits(execution, &["a", "b"]);

    attached_test_medium(&runtime)
        .flip_artifact_byte(PatchStreamPosition(2), ARTIFACT_HEADER_COMMIT_ID_OFFSET);

    assert_eq!(
        refusal_class(execution, &runtime),
        RecoveryFailureClass::CorruptCanonicalArtifact
    );
}

#[test]
fn a_gap_in_the_journal_refuses_as_non_contiguous() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);
    let runtime = runtime_with_commits(execution, &["a", "b", "c"]);

    attached_test_medium(&runtime).drop_artifact(PatchStreamPosition(2));

    assert_eq!(
        refusal_class(execution, &runtime),
        RecoveryFailureClass::NonContiguousCanonicalTail
    );
}

#[test]
fn a_repeated_entry_refuses_as_non_contiguous() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);
    let runtime = runtime_with_commits(execution, &["a", "b", "c"]);

    attached_test_medium(&runtime)
        .duplicate_artifact(PatchStreamPosition(2), PatchStreamPosition(3));

    assert_eq!(
        refusal_class(execution, &runtime),
        RecoveryFailureClass::NonContiguousCanonicalTail
    );
}

#[test]
fn a_tail_that_does_not_continue_from_the_checkpoint_refuses() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);
    let runtime = runtime_with_commits(execution, &["a", "b"]);
    checkpoint_durably(execution, &runtime).unwrap();
    for name in ["c", "d"] {
        let committed = create_entity_outcome(execution, &runtime, name);
        release_test_commit_snapshot(&runtime, &committed);
    }

    attached_test_medium(&runtime).drop_artifact(PatchStreamPosition(3));

    assert_eq!(
        refusal_class(execution, &runtime),
        RecoveryFailureClass::NonContiguousCanonicalTail
    );
}

#[test]
fn a_checkpoint_filed_under_the_wrong_position_refuses_as_corrupt() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);
    let runtime = runtime_with_commits(execution, &["a", "b"]);
    let native = runtime
        .durability_authority()
        .native_checkpoint(execution.allocation_policy())
        .unwrap();
    assert_eq!(native.covered_position(), Some(PatchStreamPosition(2)));

    attached_test_medium(&runtime).file_checkpoint_under(PatchStreamPosition(1), native);

    assert_eq!(
        refusal_class(execution, &runtime),
        RecoveryFailureClass::CorruptCheckpoint
    );
}

#[test]
fn a_damaged_checkpoint_refuses_as_corrupt() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);
    let runtime = runtime_with_commits(execution, &["a", "b"]);
    checkpoint_durably(execution, &runtime).unwrap();

    attached_test_medium(&runtime).damage_checkpoint_payload();

    assert_eq!(
        refusal_class(execution, &runtime),
        RecoveryFailureClass::CorruptCheckpoint
    );
}
