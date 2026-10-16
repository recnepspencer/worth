//! Resource/linear transfer courts only; these values claim no graph authority.
use super::*;
use worth_execution::{CancellationSource, LeaseDenial, SerialMemoryBudget, SerialRequest};

#[derive(Debug)]
struct UniqueBinding(u64); // Deliberately not Clone.

fn request(memory: SerialMemoryBudget, cancellation: &CancellationSource) -> SerialRequest {
    SerialRequest::from_memory(memory, cancellation.token(), None)
}

#[test]
fn refused_backing_returns_the_original_unique_binding_before_transfer() {
    let cancellation = CancellationSource::new();
    let memory = SerialMemoryBudget::new(0);
    let original = request(memory.clone(), &cancellation);
    let (denied, binding) = match LifecycleBinding::retain(
        &ExecutionRequest::serial(&original),
        UniqueBinding(71),
        "lifecycle fixture",
    ) {
        Err(denied) => denied,
        Ok(_) => panic!("zero memory cannot retain a nonempty binding"),
    };
    assert!(
        matches!(denied.storage_request_denial(), Some(WorkCeilingDenial::Admission(LeaseDenial::MemoryExhausted(cause))) if cause.requested > 0 && cause.admitted == 0)
    );
    assert_eq!(binding.0, 71);
    assert_eq!(memory.charged_memory_bytes(), 0);
    cancellation.cancel();
    let (denied, binding) = match LifecycleBinding::retain(
        &ExecutionRequest::serial(&original),
        binding,
        "lifecycle fixture",
    ) {
        Err(denied) => denied,
        Ok(_) => panic!("cancelled original execution cannot retain the binding"),
    };
    assert_eq!(
        denied.storage_request_denial(),
        Some(WorkCeilingDenial::Stopped(
            worth_execution::MapKernelStop::Cancelled
        ))
    );
    assert_eq!(binding.0, 71);
}

#[test]
fn original_request_funds_binding_backing_until_drop_or_consuming_transfer() {
    let cancellation = CancellationSource::new();
    let original =
        crate::test_request_policy::serial_request().with_cancellation(cancellation.token());
    let memory = original.memory().clone();
    let execution = ExecutionRequest::serial(&original);
    let first =
        LifecycleBinding::retain(&execution, UniqueBinding(81), "lifecycle fixture").unwrap();
    let baseline = 0;
    assert!(memory.charged_memory_bytes() > baseline);
    assert_eq!(first.get().0, 81);
    drop(first);
    assert_eq!(memory.charged_memory_bytes(), baseline);
    let next =
        LifecycleBinding::retain(&execution, UniqueBinding(82), "lifecycle fixture").unwrap();
    assert!(memory.charged_memory_bytes() > baseline);
    let binding = next.into_inner();
    assert_eq!(binding.0, 82);
    assert_eq!(
        memory.charged_memory_bytes(),
        baseline,
        "inline slot was consumed, not leaked"
    );
}

#[test]
fn leased_backing_keeps_original_payer_and_exact_allocation_refusal() {
    use worth_execution::{ExecutionAllocationDenialKind, ExecutionResourceLease, LeaseRequest};
    use worth_foundational::{
        DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
    };
    let cancellation = CancellationSource::new();
    let payer = |bytes| {
        ExecutionResourceLease::serial(LeaseRequest {
            policy: ExecutionRequestPolicy::new(
                ExecutionPosture::Serial,
                DeterminismContract::CanonicalBitwise,
                ExecutionBudget::new(std::num::NonZeroUsize::MIN, bytes, 1_000_000),
            ),
            deadline: None,
            cancellation: cancellation.token(),
        })
        .unwrap()
    };
    let zero = payer(0);
    let (denied, binding) = match LifecycleBinding::retain(
        &ExecutionRequest::leased(&zero),
        UniqueBinding(91),
        "leased lifecycle fixture",
    ) {
        Err(denied) => denied,
        Ok(_) => panic!("zero original payer cannot retain the lifecycle binding"),
    };
    let allocation = denied
        .allocation_denial()
        .expect("exact fixed-backing refusal");
    assert!(
        matches!(allocation.kind(), ExecutionAllocationDenialKind::Lease(LeaseDenial::MemoryExhausted(cause)) if cause.requested > 0 && cause.admitted == 0)
    );
    assert!(allocation
        .requested_payload_bytes()
        .is_some_and(|bytes| bytes > 0));
    assert_eq!(binding.0, 91);
    assert_eq!(zero.charged_memory_bytes(), 0);
    let paid = payer(64 * 1024 * 1024); // Existing generous fixture policy, not a fitted quote.
    let retained = LifecycleBinding::retain(
        &ExecutionRequest::leased(&paid),
        binding,
        "leased lifecycle fixture",
    )
    .unwrap();
    assert!(paid.charged_memory_bytes() > 0);
    assert_eq!(retained.get().0, 91);
    let binding = retained.into_inner();
    assert_eq!(binding.0, 91);
    assert_eq!(paid.charged_memory_bytes(), 0);
}
