//! Same acknowledged successor and funded historical roster survive capture retry.
use super::*;
use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};
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
