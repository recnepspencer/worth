//! Scope preparation reads, request stops and scratch admission.
use super::*;

#[test]
fn checked_custom_preparation_reads_declared_structural_state() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let target = Arc::new(OnceLock::new());
    let preparation_visits = Arc::new(AtomicUsize::new(0));
    let evaluation_visits = Arc::new(AtomicUsize::new(0));
    let runtime = RelationalRuntimeApi::builder()
        .schema_registry(crate::tests::support::test_schema_registry())
        .custom_invariant(
            CustomInvariantRegistration::new(CheckedCounterRule {
                maximum_work_units: NonZeroU64::new(100_000),
                steps: 1,
                visits: Arc::clone(&evaluation_visits),
                preparation_visits: Arc::clone(&preparation_visits),
                cancellation: None,
                preparation_steps: 2,
                preparation_scratch_bytes: 64,
                target: Some(Arc::clone(&target)),
            })
            .unwrap(),
        )
        .build();
    let entity_id = create_entity_of_kind(execution, &runtime, KindId(1), "checked-structural");
    target.set(entity_id).unwrap();
    let request = InvariantExecutionRequest::from_profile_with_contract(
        InvariantRequestProfile::CommitBoundary,
        &runtime,
        InvariantObservation::committed(runtime.storage_access().current_edition()),
        runtime.current_version_id(),
        None,
        None,
    );
    let result = InvariantEngine::new(&runtime)
        .execute(
            &worth_execution::ExecutionRequest::leased(&checked_lease(
                8 * 1024 * 1024,
                1_000_000,
                CancellationToken::new(),
            )),
            request,
        )
        .expect("checked structural preparation and evaluation succeed");
    assert_eq!(preparation_visits.load(Ordering::SeqCst), 2);
    assert_eq!(evaluation_visits.load(Ordering::SeqCst), 1);
    assert!(matches!(
        result.results()[0].verdict,
        crate::validation::data::InvariantVerdict::Pass
    ));
}

#[test]
fn checked_custom_preparation_observes_cancellation_mid_scan() {
    let cancellation = CancellationSource::new();
    let preparation_visits = Arc::new(AtomicUsize::new(0));
    let evaluation_visits = Arc::new(AtomicUsize::new(0));
    let runtime = RelationalRuntimeApi::builder()
        .schema_registry(RelationalSchemaRegistry::new())
        .custom_invariant(
            CustomInvariantRegistration::new(CheckedCounterRule {
                maximum_work_units: NonZeroU64::new(100_000),
                steps: 1,
                visits: Arc::clone(&evaluation_visits),
                preparation_visits: Arc::clone(&preparation_visits),
                cancellation: Some(cancellation.clone()),
                preparation_steps: 1_000,
                preparation_scratch_bytes: 64,
                target: None,
            })
            .unwrap(),
        )
        .build();
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
    assert_eq!(preparation_visits.load(Ordering::SeqCst), 2);
    assert_eq!(evaluation_visits.load(Ordering::SeqCst), 0);
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
        .build();
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
    // The claim is charged to the lease before it is made, so the request's
    // own memory policy refuses it: the claim alone exceeds the request.
    let Err(crate::transactions::data::TransactionCommitError::Execution {
        denial:
            crate::transactions::data::CommitExecutionDenial {
                kind:
                    crate::transactions::data::CommitExecutionDenialKind::Cause(
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

mod cancellation;
