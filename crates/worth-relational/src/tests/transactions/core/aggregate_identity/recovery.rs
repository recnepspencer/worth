use super::fixtures::*;
use crate::tests::support::*;

#[test]
fn checkpoint_schema_readmission_and_tail_replay_rederive_identical_private_records() {
    let host = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host);
    let runtime = super::fixtures::runtime();
    let born = commit(execution, &runtime, "main", birth(&[1, 2, 3, 0]));
    let members = members(&born, 4);
    runtime
        .durability_authority()
        .checkpoint(execution)
        .unwrap();
    // Burn a real canonical occurrence and append reservations after the checkpoint.
    // Recovery must use the carried occurrence and allocation gaps, not replay order.
    let discarded_batch = rewire(
        execution,
        &runtime,
        &born.snapshot,
        &[(members[1], members[0]), (members[3], members[2])],
    );
    let mut discarded = test_owner_begin_transaction_for_main(&runtime);
    discarded
        .push_batch(
            discarded_batch,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let candidate = runtime
        .prepare_branch_transaction(execution, discarded)
        .unwrap();
    runtime.discard_prepared_candidate(candidate).unwrap();
    let split = commit(
        execution,
        &runtime,
        "main",
        rewire(
            execution,
            &runtime,
            &born.snapshot,
            &[(members[1], members[0]), (members[3], members[2])],
        ),
    );
    assert_eq!(split.envelope().merged_plan.merged_intents.len(), 2);
    let expected = records(execution, &runtime, &split.snapshot);
    let plan = runtime
        .durability()
        .recovery_plan(
            execution,
            crate::durability::data::RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .unwrap();
    let mut recovered = super::fixtures::runtime();
    recovered
        .durability_recovery()
        .recover(execution, plan)
        .unwrap();
    let recovered_snapshot = snapshot_for_owner_branch(&recovered, &BranchId("main".into()));
    assert_eq!(
        records(execution, &recovered, &recovered_snapshot),
        expected
    );
    let original_envelopes = demand_canonical_history_snapshot(execution, &runtime);
    let recovered_envelopes = demand_canonical_history_snapshot(execution, &recovered);
    assert_eq!(recovered_envelopes, original_envelopes);
    recovered
        .durability_authority()
        .checkpoint(execution)
        .unwrap();
    let plan = recovered
        .durability()
        .recovery_plan(
            execution,
            crate::durability::data::RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .unwrap();
    let mut checkpointed = super::fixtures::runtime();
    checkpointed
        .durability_recovery()
        .recover(execution, plan)
        .unwrap();
    let snapshot = snapshot_for_owner_branch(&checkpointed, &BranchId("main".into()));
    assert_eq!(records(execution, &checkpointed, &snapshot), expected);
}
