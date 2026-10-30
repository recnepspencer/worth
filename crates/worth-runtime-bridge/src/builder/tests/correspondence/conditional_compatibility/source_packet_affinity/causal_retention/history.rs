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
fn execute(
    owner: &BridgeSealedRuntimeAssembly,
    lowering: &Arc<BridgeInstalledConditionalLowering>,
    execution: ExecutionRequest<'_, '_>,
    attempt: u64,
) -> BridgeConditionalCausalObservation {
    let source = crate::truth_identity_fixtures::truth_snapshot(1, 1);
    let basis = owner
        .admit_conditional_signal_basis(lowering, owner.admitted_signal_basis())
        .unwrap();
    let evidence = owner
        .execute(
            execution,
            &basis,
            BridgeConditionalExecutionRequest {
                lowering,
                query_binding_identity: "query-binding",
                query_capability_identity: 1,
                snapshot_identity: "snapshot-label",
                truth_branch_identity: None,
                bridge_snapshot_identity: Some(&source),
                execution_identity: "execution",
                attempt,
            },
            &mut (),
        )
        .unwrap();
    evidence.retain_causal_observation()
}

#[test]
fn actual_history_eviction_preserves_native_original_and_cuts_successor_custody() {
    let contacts = Arc::new(Contacts::default());
    let readers = Arc::new(AtomicUsize::new(0));
    let (owner, lowering) = owner(Arc::clone(&contacts), Arc::clone(&readers));
    assert!(owner.configure_conditional_history_for_test(true, 1));
    let serial =
        SerialRequest::from_policy(&policy(32 * 1024 * 1024), CancellationToken::new(), None);
    let memory = serial.memory().unwrap().clone();
    let baseline = memory.charged_memory_bytes();
    let request = ExecutionRequest::serial(&serial);
    let original = execute(&owner, &lowering, request, 41);
    let sibling = original.clone();
    let detail = original.history_detail().unwrap();
    assert_eq!(detail.attempt(), 41);
    assert_eq!(detail.class(), original.signal_class());
    assert_eq!(
        (detail.output_version_before(), detail.output_version()),
        original.signal_output_versions()
    );
    let first = memory.charged_memory_bytes();
    assert!(first > baseline);
    assert_eq!(readers.load(Ordering::SeqCst), 0);
    let successor = execute(&owner, &lowering, request, 42);
    assert_eq!(successor.history_detail().unwrap().attempt(), 42);
    assert_eq!(
        original.history_availability(),
        Availability::EvictedByRetention
    );
    assert_eq!(
        original.history_detail(),
        Err(Availability::EvictedByRetention)
    );
    assert_eq!(
        original.attempt(),
        41,
        "immutable Native evidence survives optional eviction"
    );
    let both = memory.charged_memory_bytes();
    assert!(both > first);
    drop((owner, lowering));
    assert_eq!(
        original.history_availability(),
        Availability::EvictedByRetention,
        "later owner closure cannot overwrite the original loss cause"
    );
    assert_eq!(successor.history_availability(), Availability::OwnerClosed);
    assert_eq!(successor.history_detail(), Err(Availability::OwnerClosed));
    drop(successor);
    assert_eq!(
        memory.charged_memory_bytes(),
        first,
        "an evicted original cannot retain its successor through the FIFO link"
    );
    drop(original);
    assert_eq!(memory.charged_memory_bytes(), first);
    assert_eq!(sibling.attempt(), 41);
    drop(sibling);
    assert_eq!(memory.charged_memory_bytes(), baseline);
    assert_eq!(readers.load(Ordering::SeqCst), 0);
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
