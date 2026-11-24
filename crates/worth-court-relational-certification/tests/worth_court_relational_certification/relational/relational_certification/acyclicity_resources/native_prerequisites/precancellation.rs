//! The whole-proposal gate retains cancellation before either stage.
use super::*;

#[test]
fn original_precancellation_prevents_both_hooks_and_an_uninterrupted_twin_completes() {
    let hooks = Arc::new(Hooks::default());
    let runtime = RelationalRuntimeApi::builder()
        .schema_registry(acyclicity_and_connectivity_registry())
        .custom_invariant(
            CustomInvariantRegistration::new(ScopeProbe {
                hooks: Arc::clone(&hooks),
                checked: true,
            })
            .unwrap(),
        )
        .build()
        .unwrap();
    let fixture = DagFixture::from_runtime(runtime, 2);
    let before = fixture.head();
    for leased in [false, true] {
        let cancellation = worth_execution::CancellationSource::new();
        let lease = request(8 * 1024 * 1024, 4_000_000, cancellation.token(), None);
        let serial =
            worth_execution::SerialRequest::from_policy(lease.policy(), cancellation.token(), None);
        cancellation.cancel();
        let execution = if leased {
            ExecutionRequest::leased(&lease)
        } else {
            ExecutionRequest::serial(&serial)
        };
        let error = fixture
            .validate(&execution, fixture.chain_batch(1))
            .unwrap_err();
        let TransactionCommitError::Execution { denial, .. } = error else {
            panic!("the original request stop is required: {error:?}");
        };
        assert_eq!(
            denial.kind,
            CommitExecutionDenialKind::Cause(RelationalExecutionDenialCause::Cancelled,)
        );
        assert_eq!(hooks.observed(), (0, 0));
        assert_eq!(fixture.head(), before);
        assert_eq!(lease.charged_memory_bytes(), 0);
        assert_eq!(serial.memory().unwrap().charged_memory_bytes(), 0);
    }
    let funded = request(8 * 1024 * 1024, 4_000_000, CancellationToken::new(), None);
    let candidate = fixture
        .validate(&ExecutionRequest::leased(&funded), fixture.chain_batch(1))
        .expect("same DAG and installed hooks complete without cancellation");
    assert_eq!(hooks.observed(), (1, 1));
    drop(candidate);
    assert_eq!(fixture.head(), before);
    assert_eq!(funded.charged_memory_bytes(), 0);
}
