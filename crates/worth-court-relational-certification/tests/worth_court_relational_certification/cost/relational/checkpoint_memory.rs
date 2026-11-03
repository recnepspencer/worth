//! Literal memory boundaries measured before specializing fragment scopes.
use worth_execution::{
    CancellationSource, ExecutionAllocationDenialClass, ExecutionRequest, LeaseRequest,
    MapKernelStop, SerialRequest, WorkCeilingDenial,
};
use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};
use worth_relational::facade::{
    durability::RelationalNativeCheckpointCaptureDenial as Denial,
    transactions::RelationalExecutionDenialCause as Cause,
};

#[test]
fn empty_checkpoint_keeps_original_memory_boundary_and_cancellation_precedence() {
    let _guard = crate::leased_preparation::ordinary_context::serial();
    // Each denial's requested/admitted values are frozen from the old path;
    // neither oracle computes scope sizes or duplicates admission logic.
    for (leased, boundary, requested, admitted) in
        [(false, 2_744, 400, 399), (true, 2_352, 384, 383)]
    {
        for (memory, cancelled) in [
            (boundary, false),
            (boundary - 1, false),
            (boundary - 1, true),
        ] {
            let source = CancellationSource::new();
            let policy = ExecutionRequestPolicy::new(
                ExecutionPosture::Serial,
                DeterminismContract::CanonicalBitwise,
                ExecutionBudget::new(std::num::NonZeroUsize::MIN, Some(memory), None),
            );
            let serial = SerialRequest::from_policy(&policy, source.token(), None);
            let lease = leased.then(|| {
                crate::leased_preparation::ordinary_context::authority()
                    .request_lease(LeaseRequest {
                        policy,
                        cancellation: source.token(),
                        deadline: None,
                    })
                    .unwrap()
            });
            let request = match &lease {
                Some(lease) => ExecutionRequest::leased(lease),
                None => ExecutionRequest::serial(&serial),
            };
            let (runtime, _) = crate::content::transaction::fixture();
            let before = runtime
                .observe_branch(&runtime.main_branch_identity())
                .unwrap()
                .0;
            if cancelled {
                source.cancel();
            }
            let result = runtime
                .durability_authority()
                .native_checkpoint(&request, request.allocation_policy());
            match (cancelled, leased, memory == boundary) {
                (false, _, true) => {
                    let checkpoint = result.unwrap();
                    assert_eq!(checkpoint.bytes().len(), 1_568);
                    drop(checkpoint);
                }
                (false, _, false) => assert!(
                    matches!(result,
                    Err(Denial::Request(Cause::PolicyMemoryExhausted {
                        requested: actual_request, admitted: actual_admitted, ancestor: 0,
                    })) if actual_request == requested && actual_admitted == admitted),
                    "{result:?}"
                ),
                (true, false, _) => {
                    assert!(matches!(result, Err(Denial::Request(Cause::Cancelled))))
                }
                (true, true, _) => match result {
                    Err(Denial::Allocation(cause)) => assert_eq!(
                        cause.classify(),
                        ExecutionAllocationDenialClass::Request(WorkCeilingDenial::Stopped(
                            MapKernelStop::Cancelled
                        ))
                    ),
                    result => panic!("original leased allocation refused cancellation: {result:?}"),
                },
            }
            assert_eq!(serial.memory().unwrap().charged_memory_bytes(), 0);
            if let Some(lease) = lease {
                assert_eq!(lease.charged_memory_bytes(), 0);
            }
            assert_eq!(
                runtime
                    .observe_branch(&runtime.main_branch_identity())
                    .unwrap()
                    .0,
                before
            );
        }
    }
}
