use super::branch_history_publications::create_entity;
use super::branch_history_publications::delete_entity;
use super::checkpoint_source_capture::capture_durably;
use super::merge_execution_observation::entity_field_display;
use worth_relational::facade::durability::RecoveryVerificationMode as RecoveryMode;
use worth_relational::facade::runtime::RelationalRuntimeApi;
use worth_testkit_relational::aspect_storage::definition::aspect_key;
use worth_testkit_relational::aspect_storage::definition::field_key;
use worth_testkit_relational::aspect_storage::publications::changed_entities;
use worth_testkit_relational::durable_medium::TestDurableMedium;
use worth_testkit_relational::durable_publication::create_entity_outcome;

#[test]
fn checkpoint_tail_preserves_replacement_and_original_record_truth() {
    let host_request = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let medium = TestDurableMedium::new();
    let runtime = RelationalRuntimeApi::builder()
        .profile(worth_relational::facade::config::RelationalRuntimeProfile::AiWorkflow)
        .schema_registry(worth_testkit_relational::durable_publication::schema())
        .build_durable(medium.attachment())
        .unwrap();
    let created = create_entity_outcome(execution, &runtime, "allocation-before");
    let original = changed_entities(&created)[0];
    assert!(runtime
        .snapshots()
        .release_snapshot(&created.snapshot)
        .is_ok());
    let deleted = delete_entity(execution, &runtime, original);
    assert!(runtime
        .snapshots()
        .release_snapshot(&deleted.snapshot)
        .is_ok());
    runtime.retention().run_pass(execution).unwrap();
    capture_durably(execution, &runtime, &medium).unwrap();

    let replacement = create_entity(execution, &runtime, "allocation-after");
    let plan = runtime
        .durability()
        .recovery_plan_from_source(
            execution,
            medium.recovery_source(),
            RecoveryMode::NormalRecoveryVerification,
        )
        .unwrap();
    let mut recovered = RelationalRuntimeApi::builder()
        .profile(worth_relational::facade::config::RelationalRuntimeProfile::AiWorkflow)
        .schema_registry(worth_testkit_relational::durable_publication::schema())
        .build_durable(medium.attachment())
        .unwrap();
    recovered
        .durability_recovery()
        .recover(execution, plan)
        .unwrap();

    let current = recovered
        .read_truth()
        .try_read_version(
            execution,
            super::branch_history_recovery::main_version(&recovered),
        )
        .expect("read request remains live")
        .expect("the selected current or historical version remains retained");
    assert_eq!(
        current
            .get_entity(replacement)
            .and_then(|record| entity_field_display(
                record,
                &aspect_key("name"),
                &field_key("name")
            )),
        Some("allocation-after".into())
    );
    let historical = recovered
        .read_truth()
        .try_read_version(execution, created.version_id)
        .expect("read request remains live")
        .expect("the selected current or historical version remains retained");
    assert_eq!(
        historical
            .get_entity(original)
            .and_then(|record| entity_field_display(
                record,
                &aspect_key("name"),
                &field_key("name")
            )),
        Some("allocation-before".into())
    );
}
