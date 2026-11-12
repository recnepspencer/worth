//! Source protection and inherited cold readers survive checkpoint reconstruction.
use super::branch_history_publications::{create_entity, create_entity_outcome_on_branch, fork};
use super::branch_history_recovery::current_head;
use std::sync::Arc;
use worth_execution::ExecutionRequest;
use worth_relational::facade::{
    durability::RecoveryVerificationMode,
    history::BranchId,
    replay::{RelationalReplayRequest, ReplayExecutionMode, ReplayVerificationMode},
    runtime::RelationalRuntime,
};
use worth_testkit_relational::{
    durable_medium::TestDurableMedium,
    durable_publication::{create_entity_outcome, durable_runtime, memory_runtime},
};
#[test]
fn recovery_plan_after_native_capture_keeps_covered_forks() {
    let request = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&request);
    let medium = TestDurableMedium::new();
    let runtime = durable_runtime(&medium);
    create_entity(execution, &runtime, "source");
    fork(execution, &runtime, "A");
    let target = create_entity_outcome_on_branch(execution, &runtime, "on-A", BranchId("A".into()));
    let native = runtime
        .durability_authority()
        .native_checkpoint(execution, execution.allocation_policy())
        .unwrap();
    let contents = medium.clone();
    contents.file_checkpoint(native);
    let selected = contents
        .recovery_source()
        .checkpoint()
        .cloned()
        .expect("actual selected native capture");
    let covered = selected.covered();
    let later = create_entity_outcome(execution, &runtime, "main-after-selected-checkpoint");
    let source: Arc<dyn worth_relational::facade::durability::RelationalDurableColdSource> =
        contents;
    let protection =
        worth_relational::facade::durability::RelationalDurableReadProtection::capture(
            &source, execution,
        )
        .unwrap();
    assert_eq!(protection.bounds().canonical.0, covered.0 + 1);
    let original =
        worth_relational::facade::durability::RelationalDurableRecoverySource::from_protected_source(
            execution,
            Some(selected),
            source,
            &protection,
        )
        .unwrap();
    let plan = runtime
        .durability()
        .recovery_plan_from_source(
            execution,
            original,
            RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .unwrap();
    let mut recovered = memory_runtime();
    recovered
        .durability_recovery()
        .recover(execution, plan)
        .unwrap();
    assert_eq!(
        current_head(&recovered, &BranchId("A".into()))
            .unwrap()
            .commit_id,
        target.commit.commit_id
    );
    assert_eq!(
        current_head(&recovered, &BranchId("main".into()))
            .unwrap()
            .commit_id,
        later.commit.commit_id
    );
}

#[test]
fn a_memory_fork_can_replay_its_compacted_durable_source() {
    let request = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&request);
    let medium = TestDurableMedium::new();
    let runtime = durable_runtime(&medium);
    create_entity(execution, &runtime, "source");
    fork(execution, &runtime, "A");
    let target = create_entity_outcome_on_branch(execution, &runtime, "on-A", BranchId("A".into()));
    runtime
        .durability_authority()
        .native_checkpoint(execution, execution.allocation_policy())
        .unwrap();
    let speculative = runtime.fork().unwrap();
    assert!(
        !speculative.durability().has_bound_durable_medium(),
        "a runtime fork cannot publish to its source's medium"
    );
    replay(execution, &speculative, target.commit.commit_id, "A");
}

fn replay(
    execution: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    commit: worth_relational::facade::history::CommitId,
    branch: &str,
) {
    let replay = runtime
        .replay_authority()
        .replay_commit(
            RelationalReplayRequest {
                commit_id: commit,
                branch_id: BranchId(branch.into()),
                execution_mode: ReplayExecutionMode::SerialDeterministic,
                verification_mode: ReplayVerificationMode::AuditRecoveryVerification,
            },
            execution,
        )
        .unwrap();
    assert!(replay.mismatches.is_empty(), "{replay:?}");
}
