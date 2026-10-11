use super::*;
use worth_foundational::{
    FoundationalBranchId, FoundationalBranchReferenceObservation, FoundationalBranchTarget,
};

fn fork_checkpoint_plan(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    mutate: fn(&mut crate::branch::RelationalBranchCellCheckpoint),
) -> RecoveryPlan {
    let runtime = persisted_runtime_with_test_schema();
    create_entity_outcome(execution, &runtime, "fork-source");
    let source = BranchId("main".to_owned());
    let (_, basis) = runtime
        .observe_fork_source(&source)
        .expect("committed main is forkable");
    runtime
        .fork_branch(BranchId("storm".to_owned()), basis)
        .expect("fork checkpoint fixture");
    runtime.durability_authority().checkpoint().unwrap();
    let mut plan = runtime
        .durability()
        .recovery_plan(RecoveryVerificationMode::NormalRecoveryVerification);
    let checkpoint = plan.checkpoint.as_mut().expect("checkpoint is selected");
    let cell = checkpoint
        .branch_cells
        .iter_mut()
        .find(|cell| cell.branch_id == BranchId("storm".to_owned()))
        .expect("forked branch cell is checkpointed");
    mutate(cell);
    plan
}

fn fork_tail_plan(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    mutate: fn(&mut crate::branch::RelationalBranchCellCheckpoint),
) -> RecoveryPlan {
    let runtime = persisted_runtime_with_test_schema();
    create_entity_outcome(execution, &runtime, "fork-source");
    let source = BranchId("main".to_owned());
    let (_, basis) = runtime
        .observe_fork_source(&source)
        .expect("committed main is forkable");
    runtime
        .fork_branch(BranchId("storm".to_owned()), basis)
        .expect("fork tail fixture");
    runtime.durability_authority().checkpoint().unwrap();
    create_entity_outcome_on_branch(
        execution,
        &runtime,
        "fork-tail",
        BranchId("storm".to_owned()),
    );
    let mut plan = runtime
        .durability()
        .recovery_plan(RecoveryVerificationMode::NormalRecoveryVerification);
    let envelope = plan
        .tail_log
        .iter_mut()
        .map(|commit| commit.envelope_mut_for_test())
        .find(|envelope| envelope.branch_context == BranchId("storm".to_owned()))
        .expect("fork tail envelope is selected");
    mutate(
        envelope
            .branch_cell_checkpoint
            .as_mut()
            .expect("tail carries an exact fork checkpoint"),
    );
    plan
}

fn assert_recovery_rejects_fork_checkpoint(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    plan: RecoveryPlan,
) {
    let mut recovered = persisted_runtime_with_test_schema();
    let error = recovered
        .durability_recovery()
        .recover(execution, plan)
        .expect_err("malformed fork provenance must fail closed");
    let error = crate::tests::support::expect_durability_failure(error);
    assert_eq!(error.class, RecoveryFailureClass::CorruptCheckpoint);
    assert_eq!(recovered.history().immutable_commit_count(), 0);
    assert!(recovered
        .history()
        .branch_head(&BranchId("main".to_owned()))
        .is_none());
}

fn provenance_only(cell: &mut crate::branch::RelationalBranchCellCheckpoint) {
    cell.fork_source_branch_id = None;
}

fn source_only(cell: &mut crate::branch::RelationalBranchCellCheckpoint) {
    cell.fork_provenance = None;
}

fn wrong_provenance_branch(cell: &mut crate::branch::RelationalBranchCellCheckpoint) {
    let source = cell
        .fork_provenance
        .as_ref()
        .expect("fork fixture has provenance")
        .clone();
    let branch_id =
        FoundationalBranchId::new(format!("relational/{}/evil", cell.runtime_instance_id))
            .expect("malformed twin remains syntactically valid");
    cell.fork_provenance = Some(FoundationalBranchReferenceObservation::new(
        branch_id,
        match source.target() {
            FoundationalBranchTarget::Empty => FoundationalBranchTarget::empty(),
            FoundationalBranchTarget::Basis(target) => {
                FoundationalBranchTarget::basis(target.clone())
            }
        },
        source.generation(),
    ));
}

#[test]
fn recovery_rejects_one_sided_fork_provenance_in_checkpoint() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    assert_recovery_rejects_fork_checkpoint(
        execution,
        fork_checkpoint_plan(execution, provenance_only),
    );
    assert_recovery_rejects_fork_checkpoint(
        execution,
        fork_checkpoint_plan(execution, source_only),
    );
}

#[test]
fn recovery_rejects_mismatched_fork_provenance_identity_in_checkpoint() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    assert_recovery_rejects_fork_checkpoint(
        execution,
        fork_checkpoint_plan(execution, wrong_provenance_branch),
    );
}

#[test]
fn recovery_rejects_one_sided_fork_provenance_in_tail() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    assert_recovery_rejects_fork_checkpoint(execution, fork_tail_plan(execution, provenance_only));
    assert_recovery_rejects_fork_checkpoint(execution, fork_tail_plan(execution, source_only));
}
