//! Fresh receipts validate the actual source and original runtime without rebuilding.
use super::*;
use worth_execution::{CancellationSource, SerialMemoryBudget, SerialRequest};
use worth_relational::facade::durability::{
    RecoveryPlan, RelationalFreshSourceAdmissionStop, RelationalRecoveryStop as Stop,
};
fn source_plan(
    runtime: &RelationalRuntime,
    source: Arc<dyn RelationalDurableColdSource>,
    execution: &ExecutionRequest<'_, '_>,
) -> RecoveryPlan {
    runtime
        .durability()
        .recovery_plan_from_source(
            execution,
            RelationalDurableRecoverySource::new(None, vec![], vec![], source),
            RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .unwrap()
}
#[test]
fn fresh_source_refuses_foreign_and_nonempty_actual_sources_without_rebuild() {
    let policy = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = ExecutionRequest::serial(&policy);
    let medium = TestDurableMedium::new();
    let mut runtime = worth_testkit_relational::durable_publication::durable_runtime(&medium);
    let foreign_medium = TestDurableMedium::new();
    let _foreign = worth_testkit_relational::durable_publication::durable_runtime(&foreign_medium);
    let original = runtime.runtime_instance_id();
    let plan = source_plan(&runtime, foreign_medium.clone(), &execution);
    assert!(runtime
        .durability_recovery()
        .admit_fresh_home_source(&execution, plan)
        .is_err());
    assert_eq!(runtime.runtime_instance_id(), original);
    let source = medium.clone();
    let stale = source_plan(&runtime, source, &execution);
    let written = create_entity_outcome(&execution, &runtime, "not-genesis");
    release_test_commit_snapshot(&runtime, &written);
    assert!(runtime
        .durability_recovery()
        .admit_fresh_home_source(&execution, stale)
        .is_err());
    assert_eq!(runtime.runtime_instance_id(), original);
    let (_, basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    assert_eq!(
        runtime
            .history()
            .branch_head_for_observation(&basis.observation())
            .unwrap()
            .as_ref(),
        Some(&written.commit)
    );
}

#[test]
fn fresh_receipt_refuses_actual_movement_and_preserves_original_cancellation() {
    let policy = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = ExecutionRequest::serial(&policy);
    let medium = TestDurableMedium::new();
    let mut runtime = worth_testkit_relational::durable_publication::durable_runtime(&medium);
    let plan = source_plan(&runtime, medium.clone(), &execution);
    let admitted = runtime
        .durability_recovery()
        .admit_fresh_home_source(&execution, plan)
        .unwrap();
    let cancelled = CancellationSource::new();
    let stopped = SerialRequest::from_memory(
        SerialMemoryBudget::new(Some(1 << 20)),
        cancelled.token(),
        None,
    );
    let stopped_execution = ExecutionRequest::serial(&stopped);
    cancelled.cancel();
    assert!(matches!(
        admitted.validate_fresh_source_runtime(&stopped_execution, &runtime),
        Err(Stop::Request(_))
    ));
    let plan = source_plan(&runtime, medium.clone(), &execution);
    assert!(matches!(
        runtime
            .durability_recovery()
            .admit_fresh_home_source(&stopped_execution, plan),
        Err(RelationalFreshSourceAdmissionStop::Recovery(Stop::Request(
            _
        )))
    ));
    let written = create_entity_outcome(&execution, &runtime, "advanced-genesis");
    release_test_commit_snapshot(&runtime, &written);
    assert!(admitted
        .validate_fresh_source_runtime(&execution, &runtime)
        .is_err());
}

#[test]
fn fresh_source_refuses_real_selected_empty_checkpoint() {
    let policy = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = ExecutionRequest::serial(&policy);
    let medium = TestDurableMedium::new();
    let mut runtime = worth_testkit_relational::durable_publication::durable_runtime(&medium);
    let selected = worth_testkit_relational::durable_publication::capture(&execution, &runtime);
    assert_eq!(selected.covered_position().unwrap().0, 0);
    medium.file_checkpoint(selected);
    let plan = runtime
        .durability()
        .recovery_plan_from_source(
            &execution,
            medium.recovery_source(),
            RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .unwrap();
    let id = runtime.runtime_instance_id();
    assert!(runtime
        .durability_recovery()
        .admit_fresh_home_source(&execution, plan)
        .is_err());
    assert_eq!(runtime.runtime_instance_id(), id);
}
