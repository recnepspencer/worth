//! Recovery readmits everything a medium hands back and refuses, before any
//! effect, whatever is damaged, missing, repeated or misfiled.

use std::sync::Arc;

use crate::durability::data::{RecoveryVerificationMode, RelationalRecoveryStop};
use crate::facade::durability::RecoveryFailureClass;
use crate::facade::history::BranchId;
use crate::publication::patch::data::PatchStreamPosition;
use crate::runtime::RelationalRuntime;
use crate::tests::support::*;

fn runtime_with_commits(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    names: &[&str],
) -> (RelationalRuntime, Arc<OwnedTestDurableMedium>) {
    let medium = OwnedTestDurableMedium::new();
    let runtime = durable_runtime_for_medium(&medium);
    for name in names {
        let committed = create_entity_outcome(execution, &runtime, name);
        release_test_commit_snapshot(&runtime, &committed);
    }
    (runtime, medium)
}

/// Plan a fresh runtime's recovery from `medium`; return the class of the
/// refusal and prove the fresh runtime is untouched.
fn refusal_class(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    medium: &Arc<OwnedTestDurableMedium>,
) -> RecoveryFailureClass {
    let recovering = durable_runtime_for_medium(&OwnedTestDurableMedium::new());
    let refused = medium
        .recovery_plan(
            execution,
            &recovering,
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
    let (_runtime, medium) = runtime_with_commits(execution, &["a", "b"]);
    let last_byte = medium.artifact_bytes(PatchStreamPosition(2)).unwrap().len() - 1;

    medium.flip_artifact_byte(PatchStreamPosition(2), last_byte);

    let error = medium.try_recovery_source().unwrap_err();
    let TestRecoverySourceRefusal::Commit(refusal) = error else {
        panic!("expected commit frame refusal: {error:?}");
    };
    assert!(matches!(
        refusal,
        crate::durability::canonical_commit_artifact::RelationalCanonicalCommitArtifactFrameRefusal::DigestMismatch { .. }
    ));
}

#[test]
fn an_artifact_that_does_not_readmit_refuses_as_corrupt() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);
    let (_runtime, medium) = runtime_with_commits(execution, &["a", "b"]);

    medium.corrupt_artifact_commit_header_with_intact_frame(PatchStreamPosition(2));

    assert_eq!(
        refusal_class(execution, &medium),
        RecoveryFailureClass::CorruptCanonicalArtifact
    );
}

#[test]
fn a_gap_in_the_journal_refuses_as_non_contiguous() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);
    let (_runtime, medium) = runtime_with_commits(execution, &["a", "b", "c"]);

    medium.drop_artifact(PatchStreamPosition(2));

    assert_eq!(
        refusal_class(execution, &medium),
        RecoveryFailureClass::NonContiguousCanonicalTail
    );
}

#[test]
fn a_repeated_entry_refuses_as_non_contiguous() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);
    let (_runtime, medium) = runtime_with_commits(execution, &["a", "b", "c"]);

    medium.duplicate_artifact(PatchStreamPosition(2), PatchStreamPosition(3));

    assert_eq!(
        refusal_class(execution, &medium),
        RecoveryFailureClass::NonContiguousCanonicalTail
    );
}

#[test]
fn a_tail_that_does_not_continue_from_the_checkpoint_refuses() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);
    let (runtime, medium) = runtime_with_commits(execution, &["a", "b"]);
    capture_and_file_checkpoint(execution, &runtime, &medium);
    for name in ["c", "d"] {
        let committed = create_entity_outcome(execution, &runtime, name);
        release_test_commit_snapshot(&runtime, &committed);
    }

    medium.drop_artifact(PatchStreamPosition(3));

    assert_eq!(
        refusal_class(execution, &medium),
        RecoveryFailureClass::NonContiguousCanonicalTail
    );
}

#[test]
fn a_checkpoint_filed_under_the_wrong_position_refuses_as_corrupt() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);
    let (runtime, medium) = runtime_with_commits(execution, &["a", "b"]);
    let native = runtime
        .durability_authority()
        .native_checkpoint_with_policy(
            execution,
            execution.allocation_policy(),
            crate::durability::data::RelationalCheckpointArtifactPolicy::ExternalMembers {
                current_members: &[],
                source: None,
            },
        )
        .unwrap();
    assert_eq!(native.covered_position(), Some(PatchStreamPosition(2)));

    let covered_transitions = native
        .covered_branch_transitions()
        .expect("a captured checkpoint knows its transition coverage");
    medium.file_checkpoint_under(PatchStreamPosition(1), covered_transitions, native);

    assert_eq!(
        refusal_class(execution, &medium),
        RecoveryFailureClass::CorruptCheckpoint
    );
}

#[test]
fn a_damaged_checkpoint_refuses_as_corrupt() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);
    let (runtime, medium) = runtime_with_commits(execution, &["a", "b"]);
    capture_and_file_checkpoint(execution, &runtime, &medium);

    medium.damage_checkpoint_payload();

    assert_eq!(
        refusal_class(execution, &medium),
        RecoveryFailureClass::CorruptCheckpoint
    );
}
