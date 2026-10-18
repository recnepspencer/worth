//! Target schema installation follows exact deferred native settlement.
use super::{
    fixture::*, recovered, RecoveredCheckpointTransitionError, RelationalOperationControl,
};

#[test]
fn deferred_codec_settlement_keeps_target_installation_with_the_exact_repair_custody() {
    let request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&request);
    let world = world(execution, true);
    let (mut target, mut recovered, receipt) = recovered(execution, &world);
    let selected = basis(&target);
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
    let transaction = target
        .durability_recovery()
        .begin_recovered_codec_transition(
            execution,
            &mut recovered,
            &selected,
            RelationalOperationControl::uninterrupted(),
            &installed,
            proof,
            activation(world.activation),
        )
        .unwrap();
    let candidate = target
        .prepare_branch_transaction(execution, transaction)
        .unwrap();
    target.durability.arm_append_failure_for_test();
    let RecoveredCheckpointTransitionError::DurabilityDeferred(deferred) = target
        .durability_recovery()
        .commit_checkpoint_transition(execution, recovered, candidate)
        .unwrap_err()
    else {
        panic!("performed codec transition must retain its native settlement route")
    };
    assert!(target
        .readmit_recovered_initial_schema_installation(receipt.clone())
        .is_err());
    let successor = basis(&target).descriptor().clone();
    let commit_count = target.history().immutable_commit_count();
    let repaired = target
        .durability_recovery()
        .repair_checkpoint_transition(*deferred)
        .unwrap();
    target
        .readmit_recovered_initial_schema_installation(receipt)
        .unwrap();
    assert_eq!(basis(&target).descriptor(), &successor);
    assert_eq!(target.history().immutable_commit_count(), commit_count);
    assert_eq!(
        protected_payload(&target, world.authored),
        protected_payload(&world.runtime, world.authored)
    );
    repaired.into_parts().1.admit_basis(basis(&target)).unwrap();
}
