//! Same acknowledged successor and funded historical roster survive capture retry.
use super::*;
use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};
use worth_query_host::facade::application_installation::ApplicationBuildIdentity;
#[test]
fn companion_post_ack_capture_refusal_repairs_same_capsule_with_historical_roster() {
    let (original, settlement) = original(model::authored_value());
    let ids = companion_ids(&settlement);
    let source_id = old::reads::source_id(&original);
    let checkpoint = old::host::capture(&original);
    let held = checkpoint.clone();
    let bytes = checkpoint.bytes().to_vec();
    let scope = request_scope();
    let capture = worth_execution::ExecutionResourceLease::serial(worth_execution::LeaseRequest {
        policy: ExecutionRequestPolicy::new(
            ExecutionPosture::Serial,
            DeterminismContract::CanonicalBitwise,
            ExecutionBudget::new(std::num::NonZeroUsize::MIN, Some(0), Some(1)),
        ),
        cancellation: scope.cancellation().execution_token(),
        deadline: scope.deadline(),
    })
    .unwrap();
    successor::output::reset_decodes();
    let mut callbacks = 0;
    let denied = transition_counted(
        checkpoint,
        installation::WorthQueryCheckpointCapturePolicy::Execution(&capture),
        &mut callbacks,
    )
    .err()
    .expect("native ACK precedes real capture exhaustion");
    let installation::WorthQueryInMemoryApplicationDenial::CheckpointTransitionCaptureStopped(
        pending,
    ) = denied
    else {
        panic!("expected acknowledged custody, got {denied:?}")
    };
    let Some(installation::WorthQueryCheckpointCaptureDenial::Allocation(cause)) =
        pending.capture_denial()
    else {
        panic!("exact native allocation cause retained")
    };
    let requested = cause.requested_payload_bytes().unwrap();
    assert!(requested > 0);
    assert_eq!(
        cause.kind(),
        worth_query_host::facade::runtime::ExecutionAllocationDenialKind::Lease(
            worth_execution::LeaseDenial::MemoryExhausted(
                worth_execution::MemoryLimitDenial::Capacity {
                    requested,
                    admitted: 0,
                    level: worth_execution::MemoryLimitLevel::Policy { ancestor: 0 }
                }
            )
        )
    );
    assert_eq!(callbacks, 1);
    assert_eq!(successor::output::decodes(), 0);
    let repaired = worth_query_host::facade::application_contribution::with_bootstrap_advancement(
        ExecutionRequestPolicy::new(
            ExecutionPosture::Serial,
            DeterminismContract::CanonicalBitwise,
            ExecutionBudget::new(
                std::num::NonZeroUsize::MIN,
                Some(64 * 1024 * 1024),
                Some(8_000_000),
            ),
        ),
        |phase| {
            // A second failed attempt returns the SAME capsule and roster.
            let pending = pending
                .repair_to_checkpoint(
                    &phase,
                    installation::WorthQueryCheckpointCapturePolicy::Execution(&capture),
                )
                .err()
                .expect("capture still unfunded");
            assert!(pending.capture_denial().is_some());
            pending
                .repair_to_checkpoint(
                    &phase,
                    installation::WorthQueryCheckpointCapturePolicy::SystemAllocation,
                )
                .unwrap()
        },
    )
    .unwrap();
    assert_eq!(callbacks, 1, "same-capsule repair never reauthors");
    assert_eq!(held.bytes(), bytes);
    let target = new::host::restore::<new::program::TargetProgram>(
        repaired,
        installation::WorthQueryApplicationProgramRoster::new(),
    );
    assert_eq!(new::reads::source_id(&target), source_id);
    assert_eq!(successor::output::decodes(), 0);
    assert_target(&target, ids);
}

#[test]
fn companion_deferred_append_cancel_and_funded_same_capsule_repair_keep_historical_roster() {
    let (original, settlement) = original(model::authored_value());
    let ids = companion_ids(&settlement);
    let source_id = old::reads::source_id(&original);
    let checkpoint = old::host::capture(&original);
    let held = checkpoint.clone();
    let bytes = checkpoint.bytes().to_vec();
    // This request places the existing receiver-owned native append fault; it
    // grants no decode budget or migration permission. Bootstrap admission and
    // every repair still run under their own original platform request.
    let placement = worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::new(Some(64 * 1024 * 1024)),
        worth_execution::CancellationSource::new().token(),
        None,
    );
    let placement = worth_execution::ExecutionRequest::serial(&placement);
    let mut callbacks = 0;
    successor::output::reset_decodes();
    let denied = installation::in_memory_rostered_program_from_checkpoint_with_transition(
        new::host::program::<new::program::TargetProgram>(),
        installation::WorthQueryApplicationProgramRoster::new(),
        successor::schema::CanonicalSchema::declaration().unwrap(),
        ((),),
        ApplicationBuildIdentity::parse("11111111-1111-1111-1111-111111111111").unwrap(),
        successor::host::limits(),
        checkpoint,
        installation::WorthQueryCheckpointProgramPredecessor::new(
            &old::host::program::<old::program::SourceProgram>()
                .revision()
                .to_string(),
        )
        .unwrap(),
        installation::WorthQueryCheckpointTransitionResources::bounded(65_536, 32, 65_536).unwrap(),
        installation::WorthQueryCheckpointCapturePolicy::SystemAllocation,
        |writer, _| {
            callbacks += 1;
            writer.fail_next_durable_append_for_test(&placement);
            Ok(())
        },
    )
    .err()
    .expect("a real native append refusal cannot expose a World");
    let installation::WorthQueryInMemoryApplicationDenial::CheckpointTransitionDeferred(pending) =
        denied
    else {
        panic!("expected actual deferred native custody: {denied:?}")
    };
    assert_eq!(callbacks, 1);
    assert_eq!(successor::output::decodes(), 0);
    assert!(pending.capture_denial().is_none());
    pending.fail_next_durable_append_for_test(&placement);
    let cancel = worth_execution::CancellationSource::new();
    let stopped = worth_execution::ExecutionResourceLease::serial(worth_execution::LeaseRequest {
        policy: ExecutionRequestPolicy::new(
            ExecutionPosture::Serial,
            DeterminismContract::CanonicalBitwise,
            ExecutionBudget::new(
                std::num::NonZeroUsize::MIN,
                Some(64 * 1024 * 1024),
                Some(8_000_000),
            ),
        ),
        cancellation: cancel.token(),
        deadline: None,
    })
    .unwrap();
    cancel.cancel();
    let pending = repair(|phase| {
        pending.repair_to_checkpoint(
            phase,
            installation::WorthQueryCheckpointCapturePolicy::Execution(&stopped),
        )
    })
    .err()
    .expect("early cancellation retains the original deferred successor and unconsumed fault");
    let Some(installation::WorthQueryCheckpointCaptureDenial::Allocation(cause)) =
        pending.capture_denial()
    else {
        panic!("exact cancellation cause retained")
    };
    assert_eq!(
        cause.kind(),
        worth_query_host::facade::runtime::ExecutionAllocationDenialKind::Cancelled
    );
    assert_eq!(cause.requested_payload_bytes(), None);
    let pending = repair(|phase| {
        pending.repair_to_checkpoint(
            phase,
            installation::WorthQueryCheckpointCapturePolicy::SystemAllocation,
        )
    })
    .err()
    .expect("the still-armed actual append failure proves deferred custody survived cancellation");
    assert!(
        pending.capture_denial().is_none(),
        "native settlement failure cannot keep a stale capture cause"
    );
    let repaired = repair(|phase| {
        pending.repair_to_checkpoint(
            phase,
            installation::WorthQueryCheckpointCapturePolicy::SystemAllocation,
        )
    })
    .expect("fresh funded repair acknowledges and captures the SAME performed successor");
    assert_eq!(callbacks, 1);
    assert_eq!(held.bytes(), bytes);
    let target = new::host::restore::<new::program::TargetProgram>(
        repaired,
        installation::WorthQueryApplicationProgramRoster::new(),
    );
    assert_eq!(new::reads::source_id(&target), source_id);
    assert_eq!(new::reads::read(&target), model::authored_value());
    assert_eq!(successor::output::decodes(), 0);
    assert_target(&target, ids);
}
fn repair<T>(
    run: impl FnOnce(
        &worth_query_host::facade::application_contribution::WorthQueryBootstrapAdvancementPhase<'_>,
    ) -> T,
) -> T {
    worth_query_host::facade::application_contribution::with_bootstrap_advancement(
        ExecutionRequestPolicy::new(
            ExecutionPosture::Serial,
            DeterminismContract::CanonicalBitwise,
            ExecutionBudget::new(
                std::num::NonZeroUsize::MIN,
                Some(64 * 1024 * 1024),
                Some(8_000_000),
            ),
        ),
        |phase| run(&phase),
    )
    .unwrap()
}
