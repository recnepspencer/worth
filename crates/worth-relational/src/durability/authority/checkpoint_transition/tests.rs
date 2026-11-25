use std::sync::Arc;

use crate::facade::durability::{
    RecoveredCheckpointTransitionDenial as Denial,
    RecoveredCheckpointTransitionError as TransitionError, RecoveredRelationalRuntimeAuthority,
};
use crate::mvcc::{PreparedRelationalCommitCandidate, RelationalPublicationOutcome};
use crate::publication::patch::data::PatchStreamPosition;
use crate::runtime::RelationalRuntime;
use crate::tests::support::*;

fn recovered_world(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> (
    RelationalRuntime,
    RecoveredRelationalRuntimeAuthority,
    Arc<OwnedTestDurableMedium>,
) {
    let medium = OwnedTestDurableMedium::new();
    let source = durable_runtime_for_medium(&medium);
    create_entity(execution, &source, "checkpoint-predecessor");
    let checkpoint = source
        .durability_authority()
        .native_checkpoint_with_policy(
            execution,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
            crate::durability::data::RelationalCheckpointArtifactPolicy::ExternalMembers {
                current_members: &[],
                source: None,
            },
        )
        .unwrap();
    // The restored runtime continues the source's medium, which already holds
    // the history its checkpoint covers.
    let mut target = durable_runtime_for_medium(&medium);
    let (_, authority) = target
        .durability_recovery()
        .restore_native_checkpoint_with_authority(
            execution,
            &checkpoint,
            crate::tests::support::durability::captured_members_source(&checkpoint),
        )
        .unwrap();
    (target, authority, medium)
}

fn candidate(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    name: &str,
) -> PreparedRelationalCommitCandidate {
    let mut transaction = test_owner_begin_transaction_for_main(runtime);
    transaction
        .push_batch(
            batch_create(name),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    runtime
        .prepare_branch_transaction(execution, transaction)
        .unwrap()
}

fn main_basis(runtime: &RelationalRuntime) -> crate::branch::AdmittedRelationalBranchBasis {
    runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap()
        .1
}

#[test]
fn refused_durable_transition_returns_its_authority_and_moves_nothing() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let (mut runtime, authority, medium) = recovered_world(execution);
    let predecessor = main_basis(&runtime);
    let count = runtime.history().immutable_commit_count();
    medium.script_next_publication(ScriptedPublication::ProvenNoEffect);
    let prepared = candidate(execution, &runtime, "refused-transition");
    let TransitionError::Refused(refusal) = runtime
        .durability_recovery()
        .commit_checkpoint_transition(execution, authority, prepared)
        .unwrap_err()
    else {
        panic!("a refused durable publication is a pre-effect transition refusal")
    };
    assert!(matches!(refusal.denial(), Denial::Publication(outcome)
    if matches!(outcome.as_ref(), RelationalPublicationOutcome::Denied(
        crate::mvcc::RelationalPublicationDenial::DurablePublicationRefused(_)
    ))));
    assert_eq!(main_basis(&runtime).descriptor(), predecessor.descriptor());
    assert_eq!(runtime.history().immutable_commit_count(), count);
    assert_eq!(runtime.visibility.published_snapshot_handle_count(), 0);
    assert_eq!(medium.head(), PatchStreamPosition(1));

    let authority = refusal.into_authority();
    let retried = candidate(execution, &runtime, "retried-transition");
    let transition = runtime
        .durability_recovery()
        .commit_checkpoint_transition(execution, authority, retried)
        .expect("the refused position was never consumed");
    assert_eq!(medium.head(), PatchStreamPosition(2));
    assert_eq!(runtime.history().immutable_commit_count(), count + 1);
    assert!(transition
        .into_parts()
        .1
        .admit_basis(main_basis(&runtime))
        .is_ok());
}

#[test]
fn unresolved_durable_transition_installs_nothing_and_names_its_fate() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let (mut runtime, authority, medium) = recovered_world(execution);
    let predecessor = main_basis(&runtime);
    let count = runtime.history().immutable_commit_count();
    medium.script_next_publication(ScriptedPublication::IndeterminateWithoutEffect);
    let prepared = candidate(execution, &runtime, "unresolved-transition");
    let TransitionError::Refused(refusal) = runtime
        .durability_recovery()
        .commit_checkpoint_transition(execution, authority, prepared)
        .unwrap_err()
    else {
        panic!("an unresolved durable publication installs no successor")
    };
    let Denial::Publication(outcome) = refusal.denial() else {
        panic!("the refusal names the publication outcome")
    };
    let RelationalPublicationOutcome::DurabilityUnresolved(fate) = outcome.as_ref() else {
        panic!("the medium could not say whether the transition is durable")
    };
    let offered = medium
        .offered_commits()
        .pop()
        .expect("the medium was asked first");
    let crate::durability::medium::RelationalDurabilityUnresolvedSubject::CanonicalCommit {
        position,
        digest,
        ..
    } = fate.subject()
    else {
        panic!("a commit's unresolved fate names the commit")
    };
    assert_eq!(position, offered.position());
    assert_eq!(digest, offered.payload_digest());
    assert_eq!(main_basis(&runtime).descriptor(), predecessor.descriptor());
    assert_eq!(runtime.history().immutable_commit_count(), count);
    assert_eq!(runtime.visibility.published_snapshot_handle_count(), 0);
}
