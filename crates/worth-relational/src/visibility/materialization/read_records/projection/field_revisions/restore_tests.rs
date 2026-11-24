use crate::durability::data::recovery_verification::RecoveryVerificationMode;
use crate::tests::support::*;
use worth_relational_durability::internal_port::durability::data::checkpoint_images::DurableCheckpointFieldsFixturePort as _;

#[test]
fn checkpoint_restore_rejects_incomplete_field_revision_column() {
    crate::branch::root::capture::with_reconstruction_fixture_for_test(|fixture_lane| {
        let host_request = worth_harness::facade::execution_fixture::serial_request();
        let execution = &worth_execution::ExecutionRequest::serial(&host_request);

        let runtime = runtime_with_test_schema();
        let created = create_entity_outcome(execution, &runtime, "same");
        runtime
            .durability_authority()
            .checkpoint(execution)
            .unwrap();
        let mut recovery = runtime
            .durability()
            .recovery_plan(
                execution,
                RecoveryVerificationMode::NormalRecoveryVerification,
            )
            .unwrap();
        let root = &mut recovery
            .checkpoint
            .as_mut()
            .unwrap()
            .branch_roots_mut_for_test()[0];
        let image = &mut root.partition_images[0].entity_arena;
        assert!(!image.field_revisions.is_empty());
        image.field_revisions.pop();
        root.partition_image_digest = crate::durability::data::branch_root_partition_image_digest(
            fixture_lane,
            &root.partition_images,
        )
        .unwrap();
        root.root_image_digest = crate::durability::data::branch_root_image_digest(
            fixture_lane,
            root.format_version,
            root.commit_id,
            root.partition_image_digest,
            root.schema_carrier_digest,
            &root.content_records,
        );

        let mut recovered = runtime_with_test_schema();
        let error = recovered
            .durability_recovery()
            .recover(execution, recovery)
            .unwrap_err();
        let error = crate::tests::support::expect_durability_failure(error);
        assert_eq!(
            error.class,
            crate::durability::data::recovery_errors::RecoveryFailureClass::CorruptCheckpoint
        );
        assert!(
            error.detail.contains("column length")
                || error.detail.contains("field revision length"),
            "{error:?}"
        );
        release_test_commit_snapshot(&runtime, &created);
    })
}
