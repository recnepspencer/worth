use super::support::collect_subscriber_patches;
use crate::tests::support::*;

#[test]
fn subscriber_cdc_resume_windows_preserve_order_across_interleaved_partitions() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_test_schema_profile(RelationalRuntimeProfile::GeometryKernel);
    let left = create_entity_in_partition(execution, &runtime, "left", PartitionId(7));
    let right = create_entity_in_partition(execution, &runtime, "right", PartitionId(11));
    let baseline_checkpoint =
        checkpoint_for_schema_version(PatchStreamPosition(2), SchemaVersionId(1));

    let updates = [
        (left, "left-1"),
        (right, "right-1"),
        (left, "left-2"),
        (right, "right-2"),
        (left, "left-3"),
        (right, "right-3"),
    ];
    for (entity, name) in updates {
        let _ = update_entity(execution, &runtime, entity, name);
    }

    let _ = create_entity_in_partition(execution, &runtime, "churn-1", PartitionId(29));
    let _ = create_entity_in_partition(execution, &runtime, "churn-2", PartitionId(31));

    let full = runtime
        .publication()
        .read_subscriber_stream(SubscriberResumeRequest::resume_after(
            baseline_checkpoint.clone(),
            64,
        ))
        .unwrap();

    for window_size in 1..=5 {
        let collected =
            collect_subscriber_patches(&runtime, baseline_checkpoint.clone(), window_size);
        assert_eq!(collected, full.patches, "window size {window_size} drifted");
    }
}
