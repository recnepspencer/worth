//! Scratch admission refuses before custom evaluation.
use super::*;

#[test]
fn checked_custom_preparation_denies_memory_before_large_scratch_claim() {
    const REQUEST_MEMORY: u64 = 128 * 1024;
    // One claim larger than the entire request cannot be funded at any posture.
    let evaluation_visits = Arc::new(AtomicUsize::new(0));
    let runtime = RelationalRuntimeApi::builder()
        .schema_registry(RelationalSchemaRegistry::new())
        .custom_invariant(
            CustomInvariantRegistration::new(CheckedCounterRule {
                maximum_work_units: NonZeroU64::new(100_000),
                steps: 1,
                visits: Arc::clone(&evaluation_visits),
                preparation_visits: Arc::new(AtomicUsize::new(0)),
                cancellation: None,
                preparation_steps: 1,
                preparation_scratch_bytes: REQUEST_MEMORY + 1,
                target: None,
            })
            .unwrap(),
        )
        .build()
        .expect("valid native runtime configuration");
    let lease = checked_lease(REQUEST_MEMORY, 10_000, CancellationToken::new());
    let request = InvariantExecutionRequest::from_profile_with_contract(
        InvariantRequestProfile::CommitBoundary,
        &runtime.invariant_validation_view(),
        InvariantObservation::committed(runtime.storage_access().current_edition()),
        runtime.current_version_id(),
        None,
        None,
    );
    let result = InvariantEngine::from_view(&runtime.invariant_validation_view())
        .execute(&worth_execution::ExecutionRequest::leased(&lease), request);
    assert_eq!(evaluation_visits.load(Ordering::SeqCst), 0);
    // The claim is charged to the lease before it is made, so the request's
    // own memory policy refuses it: the claim alone exceeds the request.
    let Err(crate::transactions::data::outcomes::transaction_commit_error::TransactionCommitError::Execution {
        denial:
            crate::transactions::data::outcomes::transaction_commit_error::CommitExecutionDenial {
                kind:
                    crate::transactions::data::outcomes::transaction_commit_error::CommitExecutionDenialKind::Cause(
                        crate::execution::RelationalExecutionDenialCause::PolicyMemoryExhausted {
                            requested,
                            admitted,
                            ..
                        },
                    ),
                ..
            },
        ..
    }) = result
    else {
        panic!("the request's memory policy must refuse the claim: {result:?}");
    };
    assert!(requested > REQUEST_MEMORY, "{requested}");
    assert!(admitted <= REQUEST_MEMORY, "{admitted}");
}
