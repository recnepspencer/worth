//! Existing reuse journey also proves original controls and honest partial work.
use super::*;
use crate::execution::RelationalExecutionDenialCause as Cause;
use crate::runtime::RelationalRetentionPassStop as Stop;
use worth_execution::{
    CancellationSource, ExecutionRequest, ExecutionWorkCeiling, SerialMemoryBudget, SerialRequest,
};

pub(super) fn entity_slot_reuse_increments_generation() {
    let memory = SerialMemoryBudget::new(None);
    let host = SerialRequest::from_memory(memory.clone(), CancellationSource::new().token(), None);
    let execution = &ExecutionRequest::serial(&host);
    let runtime = runtime_with_test_schema_profile(RelationalRuntimeProfile::AiWorkflow);
    let created = create_entity_outcome(execution, &runtime, "first");
    let original = changed_entities(&created)[0];
    release_test_commit_snapshot(&runtime, &created);
    let deleted = delete_entity(execution, &runtime, original);
    release_test_commit_snapshot(&runtime, &deleted);

    let cancelled = CancellationSource::new();
    cancelled.cancel();
    let cancelled_host = host.clone().with_cancellation(cancelled.token());
    let denied = runtime
        .retention()
        .try_run_pass(&ExecutionRequest::serial(&cancelled_host))
        .unwrap_err();
    assert_eq!(denied.stop(), Stop::Request(Cause::Cancelled));
    assert_eq!(denied.progress().slots_scanned(), 0);

    let zero_memory = SerialMemoryBudget::new(Some(0));
    let zero_host =
        SerialRequest::from_memory(zero_memory.clone(), CancellationSource::new().token(), None);
    let denied = runtime
        .retention()
        .try_run_pass(&ExecutionRequest::serial(&zero_host))
        .unwrap_err();
    assert!(
        matches!(denied.stop(), Stop::Request(Cause::PolicyMemoryExhausted {
        requested, admitted: 0, ancestor: 0,
    }) if requested > 0)
    );
    assert_eq!(denied.progress().outcome().entity_reclaimed, 0);
    assert_eq!(zero_memory.charged_memory_bytes(), 0);

    let memory_before = memory.charged_memory_bytes();
    let edition = runtime.storage_access().current_edition();
    let denied = runtime.retention().try_run_pass(execution).unwrap_err();
    assert_eq!(denied.stop(), Stop::StorageObserved);
    assert_eq!(denied.progress().outcome().entity_reclaimed, 0);
    assert!(denied.progress().slots_scanned() > 0);
    assert_eq!(memory.charged_memory_bytes(), memory_before);
    drop(edition);

    let actual = runtime.retention().try_run_pass(execution).unwrap();
    assert_eq!(actual.outcome().entity_reclaimed, 1);
    assert_eq!(
        memory.charged_memory_bytes(),
        memory_before,
        "scratch and mutation peak return to the original payer"
    );
    let replacement = create_entity(execution, &runtime, "second");
    assert_eq!(
        runtime
            .storage_access()
            .storage_stats()
            .reusable_entity_slots,
        0
    );
    assert_eq!(original.local_slot, replacement.local_slot);
    assert!(replacement.generation.0 > original.generation.0);
    completed_prefix_survives_request_stop(execution);
    worker_only_preserves_effective_original_accounting(execution);
}

fn worker_only_preserves_effective_original_accounting(setup: &ExecutionRequest<'_, '_>) {
    let policy = worth_foundational::ExecutionRequestPolicy::for_workers(
        worth_foundational::ExecutionPosture::Serial,
        worth_foundational::DeterminismContract::CanonicalBitwise,
        std::num::NonZeroUsize::MIN,
    );
    let host = SerialRequest::from_policy(&policy, CancellationSource::new().token(), None);
    let native = &ExecutionRequest::serial(&host);
    assert!(!native.has_accounting());
    let runtime = runtime_with_test_schema_profile(RelationalRuntimeProfile::AiWorkflow);
    let created = create_entity_outcome(setup, &runtime, "native-only");
    let original = changed_entities(&created)[0];
    release_test_commit_snapshot(&runtime, &created);
    let deleted = delete_entity(setup, &runtime, original);
    release_test_commit_snapshot(&runtime, &deleted);
    let pass = runtime.retention().try_run_pass(native).unwrap();
    assert_eq!(pass.outcome().entity_reclaimed, 1);
    let replacement = create_entity(setup, &runtime, "native-replacement");
    assert_eq!(original.local_slot, replacement.local_slot);
    assert!(replacement.generation.0 > original.generation.0);

    let inherited = retired_world(setup, 2);
    let parent_memory = SerialMemoryBudget::new(Some(u64::MAX));
    let parent = SerialRequest::from_memory(
        parent_memory.clone(),
        CancellationSource::new().token(),
        None,
    );
    let mut observed = None;
    let outer = ExecutionRequest::serial(&parent).run_native_with_checkpoint(|context| {
        assert!(context.has_accounting());
        assert!(
            !native.has_accounting(),
            "the backing flag cannot decide ancestry"
        );
        // Leave the SDK's exact first frame so optional_stage can inspect the
        // effective accounted context. Its next funded-stage admission refuses;
        // this does not identify a particular later metadata buffer.
        let frame = ExecutionWorkCeiling::scope_memory_bound_for_test(None, Some(&host)).unwrap();
        let remaining = parent_memory
            .limit()
            .unwrap()
            .checked_sub(parent_memory.charged_memory_bytes())
            .and_then(|remaining| remaining.checked_sub(frame))
            .unwrap();
        let held = parent_memory.reserve(remaining).unwrap();
        observed = Some(inherited.retention().try_run_pass(native));
        drop(held);
    });
    let denied = observed
        .expect("actual original child executes")
        .unwrap_err();
    assert!(
        matches!(denied.stop(), Stop::Request(Cause::PolicyMemoryExhausted {
        requested, admitted: 0, ancestor: 0,
    }) if requested > 0)
    );
    assert_eq!(denied.progress().outcome().entity_reclaimed, 0);
    assert_eq!(
        inherited
            .storage_access()
            .storage_stats()
            .reusable_entity_slots,
        0
    );
    assert_eq!(parent_memory.charged_memory_bytes(), 0);
    assert!(matches!(
        outer,
        Err(worth_execution::WorkCeilingDenial::Stopped(
            worth_execution::MapKernelStop::NestedStopped
        ))
    ));
}

fn retired_world(execution: &ExecutionRequest<'_, '_>, batch: usize) -> RelationalRuntime {
    let runtime = RelationalRuntimeApi::builder()
        .profile(RelationalRuntimeProfile::AiWorkflow)
        .schema_registry(test_schema_registry())
        .mvcc(MvccConfig {
            track_visibility_metadata: true,
            snapshot_release_policy: SnapshotReleasePolicy::ExplicitRelease,
            auto_reclaim_deleted_records: true,
            reclaim_batch_size: batch,
            retention_backend: RetentionBackend::PinTrackedRetention,
        })
        .build();
    let first = create_entity_outcome(execution, &runtime, "one");
    let second = create_entity_outcome(execution, &runtime, "two");
    let originals = [changed_entities(&first)[0], changed_entities(&second)[0]];
    release_test_commit_snapshot(&runtime, &first);
    release_test_commit_snapshot(&runtime, &second);
    for entity in originals {
        let deleted = delete_entity(execution, &runtime, entity);
        release_test_commit_snapshot(&runtime, &deleted);
    }
    runtime
}

fn completed_prefix_survives_request_stop(execution: &ExecutionRequest<'_, '_>) {
    // Identical authored records and scan shape; only the lawful batch policy
    // changes. Use its real request report once, without fitting a threshold.
    let one_batch = retired_world(execution, 1);
    let (first, report) = execution
        .run(ExecutionWorkCeiling::new(None), |_| {
            one_batch.retention().try_run_pass(execution)
        })
        .unwrap();
    assert_eq!(first.unwrap().outcome().entity_reclaimed, 1);
    let full_batch = retired_world(execution, 2);
    let mut retained_stop = None;
    let result = execution.run(
        ExecutionWorkCeiling::new(Some(report.charged_work())),
        |_| {
            let result = full_batch.retention().try_run_pass(execution);
            retained_stop = Some(result);
        },
    );
    assert!(
        result.is_ok(),
        "the SDK returns the callback value after its inner pattern reports a typed stop"
    );
    let stopped = retained_stop
        .expect("the public pass returns its completed prefix")
        .unwrap_err();
    assert_eq!(stopped.stop(), Stop::Request(Cause::WorkExhausted));
    let reclaimed = stopped.progress().outcome().entity_reclaimed;
    assert!(
        reclaimed > 0 && reclaimed < 2,
        "request actually interrupts after a handoff"
    );
    assert_eq!(
        full_batch
            .storage_access()
            .storage_stats()
            .reusable_entity_slots,
        reclaimed
    );
    let fresh_host = crate::test_execution_policy::serial_request();
    let retry = full_batch
        .retention()
        .try_run_pass(&ExecutionRequest::serial(&fresh_host))
        .unwrap();
    assert_eq!(retry.outcome().entity_reclaimed + reclaimed, 2);
    assert_eq!(
        full_batch
            .storage_access()
            .storage_stats()
            .reusable_entity_slots,
        2
    );
}
