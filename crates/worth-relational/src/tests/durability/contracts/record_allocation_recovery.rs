use super::*;
use crate::durability::data::RecoveryVerificationMode as RecoveryMode;

#[test]
fn tail_replay_consumes_canonical_reused_record_allocation() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = durable_runtime_with_test_schema_profile(
        crate::facade::config::RelationalRuntimeProfile::AiWorkflow,
    );
    let created = create_entity_outcome(execution, &runtime, "allocation-before");
    let original = changed_entities(&created)[0];
    assert!(runtime
        .visibility_authority()
        .release_snapshot(&created.snapshot)
        .is_ok());
    let deleted = delete_entity(execution, &runtime, original);
    assert!(runtime
        .visibility_authority()
        .release_snapshot(&deleted.snapshot)
        .is_ok());
    assert!(
        runtime
            .retention()
            .run_pass(execution)
            .unwrap()
            .entity_reclaimed
            <= 1
    );
    assert_eq!(
        runtime
            .storage_access()
            .storage_stats(execution)
            .unwrap()
            .reusable_entity_slots,
        1
    );
    crate::tests::support::checkpoint_durably(execution, &runtime).unwrap();

    let replacement = create_entity(execution, &runtime, "allocation-after");
    let plan = crate::tests::support::recovery_plan_for(
        execution,
        &runtime,
        RecoveryMode::NormalRecoveryVerification,
    )
    .unwrap();
    let mut recovered = durable_runtime_with_test_schema_profile(
        crate::facade::config::RelationalRuntimeProfile::AiWorkflow,
    );
    bind_durable_medium(&mut recovered, attached_test_medium(&runtime).attachment());
    recovered
        .durability_recovery()
        .recover(execution, plan)
        .unwrap();

    assert_eq!(replacement.local_slot, original.local_slot);
    assert!(replacement.generation.0 > original.generation.0);
    let current = recovered
        .read_truth()
        .read_version(execution, recovered.current_version_id())
        .expect("read request remains live");
    assert_eq!(
        read_entity_name(current.get_entity(replacement).unwrap()),
        Some("allocation-after".into())
    );
    let historical = recovered
        .read_truth()
        .read_version(execution, created.version_id)
        .expect("read request remains live");
    assert_eq!(
        read_entity_name(historical.get_entity(original).unwrap()),
        Some("allocation-before".into())
    );
}
