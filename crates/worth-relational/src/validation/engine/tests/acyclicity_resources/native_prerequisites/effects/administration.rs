//! Existing unchecked-kernel admission precedes native domain adjudication.
use super::*;

#[test]
fn unchecked_automatic_admission_retains_precedence_over_a_real_native_cycle() {
    let hooks = Arc::new(Hooks::default());
    let runtime = RelationalRuntimeApi::builder()
        .schema_registry(acyclicity_and_connectivity_registry())
        .custom_invariant(
            CustomInvariantRegistration::new(AtPoint {
                probe: ScopeProbe(Arc::clone(&hooks)),
                point: Point::CommitBoundary,
                checked: false,
            })
            .unwrap(),
        )
        .build();
    let fixture = DagFixture::from_runtime(runtime, 2);
    let original = crate::test_execution_policy::serial_request();
    let seed = fixture
        .publish(
            &ExecutionRequest::serial(&original),
            fixture.edge_batch(fixture.nodes[1], fixture.nodes[0], "reverse"),
        )
        .unwrap();
    crate::tests::support::release_test_commit_snapshot(&fixture.runtime, &seed);
    assert_eq!(hooks.observed(), (1, 1));
    let before = fixture.head();
    let automatic = crate::tests::support::test_execution_authority()
        .request_lease(LeaseRequest {
            policy: ExecutionRequestPolicy::new(
                ExecutionPosture::Automatic,
                DeterminismContract::CanonicalBitwise,
                ExecutionBudget::new(
                    std::num::NonZeroUsize::new(4).unwrap(),
                    Some(8 * 1024 * 1024),
                    Some(4_000_000),
                ),
            ),
            cancellation: CancellationToken::new(),
            deadline: None,
        })
        .unwrap();
    let failure = fixture
        .validate(
            &ExecutionRequest::leased(&automatic),
            fixture.edge_batch(fixture.nodes[0], fixture.nodes[1], "closing"),
        )
        .unwrap_err();
    let TransactionCommitError::Execution { denial, .. } = failure else {
        panic!("metadata admission retains its original priority");
    };
    assert_eq!(
        denial.kind,
        crate::transactions::data::CommitExecutionDenialKind::Cause(
            crate::execution::RelationalExecutionDenialCause::UncheckedCustomKernel,
        )
    );
    assert_eq!(denial.partition_identity, None);
    let serial = super::super::super::fixture::request(
        8 * 1024 * 1024,
        4_000_000,
        CancellationToken::new(),
        None,
    );
    let failure = fixture
        .validate(
            &ExecutionRequest::leased(&serial),
            fixture.edge_batch(fixture.nodes[0], fixture.nodes[1], "closing"),
        )
        .unwrap_err();
    let TransactionCommitError::Conflict { error, .. } = failure else {
        panic!("serial admission reaches native adjudication");
    };
    assert!(matches!(&error.class, ConflictClass::InvariantViolation {
        fields: InvariantViolationFields::Acyclicity { contract_id, .. }, ..
    } if contract_id.as_str() == "no_cycles"));
    assert_eq!(hooks.observed(), (1, 1));
    assert_eq!(fixture.head(), before);
    assert_eq!(automatic.charged_memory_bytes(), 0);
    assert_eq!(serial.charged_memory_bytes(), 0);
}
