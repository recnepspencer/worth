//! The complete relation closure keeps original refusal and exact repair custody.
use super::{
    super::fixture::{activation, activation_proof, basis, capture, protected_payload},
    fixture::*,
};
use crate::durability::authority::RecoveredCheckpointTransitionError;
use crate::mvcc::RelationalOperationControl;

#[test]
fn closed_relation_insufficient_funding_preserves_source_for_fresh_funded_retry() {
    let request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&request);
    let world = world(execution);
    let (mut target, mut recovered, _) = recover(execution, &world);
    let selected = basis(&target);
    let before = capture(&target);
    let (owner, installed) = target
        .durability_recovery()
        .issue_recovered_codec_source_publication_owner(execution, &recovered)
        .unwrap();
    let proof = activation_proof(
        execution,
        &mut target,
        &recovered,
        &selected,
        &owner,
        world.activation,
    );
    let limited = worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::new(1),
        worth_execution::CancellationSource::new().token(),
        None,
    );
    let limited_execution = worth_execution::ExecutionRequest::serial(&limited);
    let denied = target
        .durability_recovery()
        .begin_recovered_codec_transition(
            &limited_execution,
            &mut recovered,
            &selected,
            RelationalOperationControl::uninterrupted(),
            &installed,
            proof,
            activation(world.activation),
        )
        .unwrap_err();
    assert!(
        denied.request_cause().is_some(),
        "actual native admission must refuse funding: {denied:?}"
    );
    assert!(selected.is_current());
    assert_eq!(capture(&target), before);
    let candidate = candidate(
        execution,
        &mut target,
        &mut recovered,
        world.activation,
        RelationalOperationControl::uninterrupted(),
        None,
    )
    .unwrap();
    target
        .durability_recovery()
        .commit_checkpoint_transition(execution, recovered, candidate)
        .unwrap()
        .into_parts()
        .1
        .admit_basis(basis(&target))
        .unwrap();
    assert_eq!(
        protected_payload(&target, world.authored),
        protected_payload(&world.runtime, world.authored)
    );
}

#[test]
fn closed_relation_cancellation_refuses_before_effects_then_exact_retry_can_defer_and_repair() {
    let request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&request);
    let world = world(execution);
    let (mut target, mut recovered, receipt) = recover(execution, &world);
    let selected = basis(&target);
    let before = capture(&target);
    let cancellation = crate::runtime::RelationalCancellationSource::new();
    let candidate = candidate(
        execution,
        &mut target,
        &mut recovered,
        world.activation,
        cancellation.token().into(),
        None,
    )
    .unwrap();
    cancellation.cancel();
    let RecoveredCheckpointTransitionError::Refused(refused) = target
        .durability_recovery()
        .commit_checkpoint_transition(execution, recovered, candidate)
        .unwrap_err()
    else {
        panic!("cancel before effects")
    };
    let mut recovered = refused.into_authority();
    assert!(selected.is_current());
    assert_eq!(capture(&target), before);
    assert_order(&target, &world.outputs, &world.relations);
    let candidate = super::fixture::candidate(
        execution,
        &mut target,
        &mut recovered,
        world.activation,
        RelationalOperationControl::uninterrupted(),
        None,
    )
    .unwrap();
    target.durability.arm_append_failure_for_test();
    let RecoveredCheckpointTransitionError::DurabilityDeferred(deferred) = target
        .durability_recovery()
        .commit_checkpoint_transition(execution, recovered, candidate)
        .unwrap_err()
    else {
        panic!("performed successor keeps settlement custody")
    };
    let successor = basis(&target).descriptor().clone();
    let commits = target.history().immutable_commit_count();
    assert!(target
        .readmit_recovered_initial_schema_installation(receipt.clone())
        .is_err());
    let repaired = target
        .durability_recovery()
        .repair_checkpoint_transition(*deferred)
        .unwrap();
    target
        .readmit_recovered_initial_schema_installation(receipt)
        .unwrap();
    assert_eq!(basis(&target).descriptor(), &successor);
    assert_eq!(target.history().immutable_commit_count(), commits);
    assert_eq!(
        protected_payload(&target, world.authored),
        protected_payload(&world.runtime, world.authored)
    );
    repaired.into_parts().1.admit_basis(basis(&target)).unwrap();
}
