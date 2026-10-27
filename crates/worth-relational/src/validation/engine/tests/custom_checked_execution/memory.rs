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
        &runtime,
        InvariantObservation::committed(runtime.storage_access().current_edition()),
        runtime.current_version_id(),
        None,
        None,
    );
    let result = InvariantEngine::new(&runtime)
        .execute(&worth_execution::ExecutionRequest::leased(&lease), request);
    assert_eq!(evaluation_visits.load(Ordering::SeqCst), 0);
    assert!(matches!(
        result,
        Err(
            crate::transactions::data::TransactionCommitError::Execution {
                denial: crate::transactions::data::CommitExecutionDenial {
                    kind: crate::transactions::data::CommitExecutionDenialKind::Cause(
                        crate::execution::RelationalExecutionDenialCause::ScratchCapacityExceeded
                    ),
                    ..
                },
                ..
            }
        )
    ));
}
