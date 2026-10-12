use super::*;

#[test]
fn checked_custom_evaluation_observes_cancellation_during_structural_views() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let cancellation = CancellationSource::new();
    let target = Arc::new(OnceLock::new());
    let visits = Arc::new(AtomicUsize::new(0));
    let runtime = RelationalRuntimeApi::builder()
        .schema_registry(crate::tests::support::test_schema_registry())
        .custom_invariant(
            CustomInvariantRegistration::new(CheckedCounterRule {
                steps: 1_000,
                visits: Arc::clone(&visits),
                preparation_visits: Arc::new(AtomicUsize::new(0)),
                cancellation: Some(cancellation.clone()),
                preparation_steps: 0,
                preparation_scratch_bytes: 0,
                target: Some(Arc::clone(&target)),
            })
            .unwrap(),
        )
        .build();
    target
        .set(create_entity_of_kind(
            execution,
            &runtime,
            KindId(1),
            "checked-cancel",
        ))
        .unwrap();
    let lease = checked_lease(8 * 1024 * 1024, 10_000, cancellation.token());
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
    assert_eq!(visits.load(Ordering::SeqCst), 2);
    assert!(matches!(
        result,
        Err(
            crate::transactions::data::TransactionCommitError::Execution {
                denial: crate::transactions::data::CommitExecutionDenial {
                    kind: crate::transactions::data::CommitExecutionDenialKind::Cause(
                        crate::execution::RelationalExecutionDenialCause::Cancelled
                    ),
                    ..
                },
                ..
            }
        )
    ));
}
