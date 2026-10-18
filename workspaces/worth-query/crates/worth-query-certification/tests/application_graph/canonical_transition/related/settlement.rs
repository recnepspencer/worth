//! Actual post-ACK capture refusal repairs the same complete graph successor.
use super::*;
use crate::document_retention_model::operator_identity::request_scope;
use std::num::NonZeroUsize;
use worth_execution::LeaseRequest;
use worth_foundational::{ExecutionBudget, ExecutionRequestPolicy};
use worth_query_host::facade::{
    application_installation as installation, runtime::ExecutionAllocationDenialKind,
};
#[test]
fn related_acknowledged_successor_repairs_capture_without_reauthoring_or_old_decode() {
    let original = host::install(model::authored_value()).unwrap();
    let _old = host::produce(&original);
    let source_id = host::source_id(&original);
    let checkpoint = host::capture(&original);
    let retained = checkpoint.clone();
    let bytes = checkpoint.bytes().to_vec();
    let scope = request_scope();
    let mut callbacks = 0;
    successor::output::reset_decodes();
    // The existing native serial backing owns only this capture allowance.
    // It grants no source, snapshot or transition authority, and stays outside
    // bootstrap advancement: opening another bootstrap inside a live phase denies.
    let capture = worth_execution::ExecutionResourceLease::serial(LeaseRequest {
        policy: ExecutionRequestPolicy::new(
            worth_foundational::ExecutionPosture::Serial,
            worth_foundational::DeterminismContract::CanonicalBitwise,
            ExecutionBudget::new(NonZeroUsize::MIN, 0, 1),
        ),
        cancellation: scope.cancellation().execution_token(),
        deadline: scope.deadline(),
    })
    .unwrap();
    let denied = installation::in_memory_rostered_program_from_checkpoint_with_transition(
        successor::host::program(),
        installation::WorthQueryApplicationProgramRoster::new().support(
            successor::companion::host::program::<successor::companion::program::TargetProgram>(),
        ),
        successor::schema::CanonicalSchema::declaration().unwrap(),
        ((),),
        successor::host::limits(),
        checkpoint,
        installation::WorthQueryCheckpointProgramPredecessor::new(
            &host::program().revision().to_string(),
        )
        .unwrap(),
        installation::WorthQueryCheckpointTransitionResources::bounded(8192, 32, 65_536).unwrap(),
        installation::WorthQueryCheckpointCapturePolicy::Execution(&capture),
        |_, _| {
            callbacks += 1;
            Ok(())
        },
    )
    .err()
    .expect("zero capture memory stops after native acknowledgement");
    let installation::WorthQueryInMemoryApplicationDenial::CheckpointTransitionCaptureStopped(
        pending,
    ) = denied
    else {
        panic!("exact acknowledged custody must survive capture: {denied:?}")
    };
    assert_eq!(callbacks, 1);
    assert_eq!(successor::output::decodes(), 0);
    let Some(installation::WorthQueryCheckpointCaptureDenial::Allocation(cause)) =
        pending.capture_denial()
    else {
        panic!("capture refusal remains precise")
    };
    let requested = cause.requested_payload_bytes().unwrap();
    assert!(requested > 0);
    assert_eq!(
        cause.kind(),
        ExecutionAllocationDenialKind::Lease(worth_execution::LeaseDenial::MemoryExhausted(
            worth_execution::MemoryLimitDenial {
                requested,
                admitted: 0,
                level: worth_execution::MemoryLimitLevel::Policy { ancestor: 0 },
            }
        ))
    );
    let repaired = worth_query_host::facade::application_contribution::with_bootstrap_advancement(
        ExecutionRequestPolicy::new(
            worth_foundational::ExecutionPosture::Serial,
            worth_foundational::DeterminismContract::CanonicalBitwise,
            ExecutionBudget::new(NonZeroUsize::MIN, 64 * 1024 * 1024, 8_000_000),
        ),
        |phase| {
            pending
                .repair_to_checkpoint(
                    &phase,
                    installation::WorthQueryCheckpointCapturePolicy::SystemAllocation,
                )
                .unwrap()
        },
    )
    .unwrap();
    assert_eq!(callbacks, 1);
    assert_eq!(retained.bytes(), bytes);
    let target = successor::host::restore(repaired);
    assert_eq!(successor::host::source_id(&target), source_id);
    assert_eq!(successor::output::decodes(), 0);
    let _new = successor::host::produce(&target);
    journey::assert_target_pages(&target, false);
}
