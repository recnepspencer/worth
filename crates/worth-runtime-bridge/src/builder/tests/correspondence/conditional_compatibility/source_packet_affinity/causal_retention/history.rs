//! Real execution histories can be lost without destroying original Native evidence.
use super::*;
use crate::facade::{
    BridgeConditionalCausalObservation, BridgeConditionalHistoryAvailability as Availability,
};
use worth_execution::*;

fn policy(bytes: u64) -> worth_foundational::ExecutionRequestPolicy {
    worth_foundational::ExecutionRequestPolicy::new(
        worth_foundational::ExecutionPosture::Serial,
        worth_foundational::DeterminismContract::CanonicalBitwise,
        worth_foundational::ExecutionBudget::new(
            std::num::NonZeroUsize::MIN,
            Some(bytes),
            Some(100_000_000),
        ),
    )
}
#[test]
fn real_optional_capacity_denial_is_tolerated_but_request_stop_is_preserved() {
    let contacts = Arc::new(Contacts::default());
    let readers = Arc::new(AtomicUsize::new(0));
    let (owner, lowering) = owner(Arc::clone(&contacts), readers);
    assert!(owner.configure_conditional_history_for_test(false, 1));
    let serial =
        SerialRequest::from_policy(&policy(32 * 1024 * 1024), CancellationToken::new(), None);
    let execution = ExecutionRequest::serial(&serial);
    let source = crate::truth_identity_fixtures::truth_snapshot(1, 1);
    let basis = owner
        .admit_conditional_signal_basis(&lowering, owner.admitted_signal_basis())
        .unwrap();
    let evidence = owner
        .execute(
            execution,
            &basis,
            BridgeConditionalExecutionRequest {
                lowering: &lowering,
                query_binding_identity: "query-binding",
                query_capability_identity: 1,
                snapshot_identity: "snapshot-label",
                truth_branch_identity: None,
                bridge_snapshot_identity: Some(&source),
                execution_identity: "execution",
                attempt: 1,
            },
            &mut (),
        )
        .unwrap();
    assert_eq!(
        evidence.retain_causal_observation().history_availability(),
        Availability::OmittedByTier
    );
    // Exercise the production capture admission with a genuine already executed decision.
    // No constructed label, digest or callback result can become Native evidence.
    let mut history =
        crate::diagnostics::conditional_history::ConditionalExecutionHistory::default();
    let held = serial.memory().unwrap().reserve(32 * 1024 * 1024).unwrap();
    let denied = history
        .capture(execution, evidence.signal(), true, 1)
        .unwrap();
    assert!(matches!(
        denied.availability(),
        Availability::CaptureDenied(MemoryLimitDenial::Capacity { .. })
    ));
    assert!(denied.detail().is_err());
    drop(held);
    assert_eq!(execution.consult(), Ok(()));
    let admitted = history
        .capture(execution, evidence.signal(), true, 1)
        .unwrap();
    assert_eq!(admitted.detail().unwrap().attempt(), 1);
    let charged = serial.memory().unwrap().charged_memory_bytes();
    assert!(charged > 0);
    let cancellation = CancellationSource::new();
    let cancelled =
        SerialRequest::from_policy(&policy(32 * 1024 * 1024), cancellation.token(), None);
    cancellation.cancel();
    assert!(matches!(
        history.capture(
            ExecutionRequest::serial(&cancelled),
            evidence.signal(),
            true,
            1
        ),
        Err(WorkCeilingDenial::Stopped(MapKernelStop::Cancelled))
    ));
    assert_eq!(serial.memory().unwrap().charged_memory_bytes(), charged);
    drop((history, admitted, denied));
    assert_eq!(serial.memory().unwrap().charged_memory_bytes(), 0);
    assert_eq!(evidence.attempt(), 1);
}
