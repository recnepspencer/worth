use super::super::support::collect_subscriber_patches;
use crate::facade::publication::SubscriberResumeRequest;
use crate::tests::support::*;

#[test]
fn cdc_certification_snapshot_pinning_is_neutral_under_rewrite_churn() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let pinned_runtime = runtime_with_test_schema_profile(RelationalRuntimeProfile::GeometryKernel);
    let unpinned_runtime =
        runtime_with_test_schema_profile(RelationalRuntimeProfile::GeometryKernel);

    let pinned_left =
        create_entity_in_partition(execution, &pinned_runtime, "baseline-left", PartitionId(7));
    let pinned_right = create_entity_in_partition(
        execution,
        &pinned_runtime,
        "baseline-right",
        PartitionId(11),
    );
    let unpinned_left = create_entity_in_partition(
        execution,
        &unpinned_runtime,
        "baseline-left",
        PartitionId(7),
    );
    let unpinned_right = create_entity_in_partition(
        execution,
        &unpinned_runtime,
        "baseline-right",
        PartitionId(11),
    );
    let baseline_checkpoint =
        checkpoint_for_schema_version(PatchStreamPosition(2), SchemaVersionId(1));
    let pinned_snapshot = pinned_runtime.visibility_authority().snapshot();

    for step in 0..48 {
        let left_name = format!("left-rewrite-{step}");
        let right_name = format!("right-rewrite-{step}");
        let churn_name = format!("churn-{step}");

        update_entity_and_release_snapshot(execution, &pinned_runtime, pinned_left, &left_name);
        update_entity_and_release_snapshot(execution, &pinned_runtime, pinned_right, &right_name);
        update_entity_and_release_snapshot(execution, &unpinned_runtime, unpinned_left, &left_name);
        update_entity_and_release_snapshot(
            execution,
            &unpinned_runtime,
            unpinned_right,
            &right_name,
        );

        if step % 3 == 0 {
            let partition = match step % 4 {
                0 => PartitionId(7),
                1 => PartitionId(11),
                2 => PartitionId(29),
                _ => PartitionId(31),
            };
            let _ = create_entity_in_partition(execution, &pinned_runtime, &churn_name, partition);
            let _ =
                create_entity_in_partition(execution, &unpinned_runtime, &churn_name, partition);
        }
    }

    let pinned_full = collect_subscriber_patches(&pinned_runtime, baseline_checkpoint.clone(), 512);
    let unpinned_full =
        collect_subscriber_patches(&unpinned_runtime, baseline_checkpoint.clone(), 512);
    assert_eq!(pinned_full, unpinned_full);

    for window_size in [1_usize, 2, 3, 5, 8, 13] {
        let pinned =
            collect_subscriber_patches(&pinned_runtime, baseline_checkpoint.clone(), window_size);
        let unpinned =
            collect_subscriber_patches(&unpinned_runtime, baseline_checkpoint.clone(), window_size);
        assert_eq!(pinned, pinned_full, "pinned window {window_size} drifted");
        assert_eq!(
            unpinned, unpinned_full,
            "unpinned window {window_size} drifted"
        );
        assert_eq!(
            pinned, unpinned,
            "window {window_size} diverged under pinning"
        );
    }

    let pinned_batch = pinned_runtime
        .publication()
        .read_subscriber_stream(SubscriberResumeRequest::resume_after(
            baseline_checkpoint,
            8,
        ))
        .unwrap();
    let unpinned_batch = unpinned_runtime
        .publication()
        .read_subscriber_stream(SubscriberResumeRequest::resume_after(
            checkpoint_for_schema_version(PatchStreamPosition(2), SchemaVersionId(1)),
            8,
        ))
        .unwrap();
    assert_eq!(
        pinned_batch.recovery_decision,
        unpinned_batch.recovery_decision
    );

    let latest_snapshot = pinned_runtime.visibility_authority().snapshot();
    let pinned_snapshot_read = pinned_runtime
        .read_truth()
        .read_snapshot(&pinned_snapshot, execution)
        .expect("read request remains live")
        .unwrap();
    let pinned_latest_read = pinned_runtime
        .read_truth()
        .read_snapshot(&latest_snapshot, execution)
        .expect("read request remains live")
        .unwrap();

    assert_eq!(
        read_entity_name(pinned_snapshot_read.get_entity(pinned_left).unwrap()),
        Some("baseline-left".into())
    );
    assert_eq!(
        read_entity_name(pinned_snapshot_read.get_entity(pinned_right).unwrap()),
        Some("baseline-right".into())
    );
    assert_eq!(
        read_entity_name(pinned_latest_read.get_entity(pinned_left).unwrap()),
        Some("left-rewrite-47".into())
    );
    assert_eq!(
        read_entity_name(pinned_latest_read.get_entity(pinned_right).unwrap()),
        Some("right-rewrite-47".into())
    );

    let retention = pinned_runtime.retention().inspect_plan(execution).unwrap();
    assert!(retention.active_snapshot_count >= 2);
    assert_eq!(retention.snapshot_pinned_entities, 0);
    assert!(pinned_runtime
        .visibility_authority()
        .release_snapshot(&pinned_snapshot)
        .is_ok());
    assert!(pinned_runtime
        .visibility_authority()
        .release_snapshot(&latest_snapshot)
        .is_ok());
}
