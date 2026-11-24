//! Scratch admission refuses before custom evaluation.
use super::*;

#[test]
fn checked_custom_preparation_denies_memory_before_large_scratch_claim() {
    const REQUEST_MEMORY: u64 = 128 * 1024;
    // One claim larger than the entire request cannot be funded at any posture.
    let evaluation_visits = Arc::new(AtomicUsize::new(0));
    let instrumentation = worth_relational_branch::internal_port::runtime::state::subsystems::services::instrumentation::RuntimeInstrumentation::default();
    let version = crate::contracts::identity::data::primitives::VersionId(0);
    let mut config =
        crate::contracts::config::data::runtime_config::RelationalRuntimeConfig::default();
    config.schema.registry = RelationalSchemaRegistry::new();
    let view = crate::validation::engine::test_view::invariant_view_with_custom_rules(
        &instrumentation,
        config,
        version,
        vec![CustomInvariantRegistration::new(CheckedCounterRule {
            maximum_work_units: NonZeroU64::new(100_000),
            steps: 1,
            visits: Arc::clone(&evaluation_visits),
            preparation_visits: Arc::new(AtomicUsize::new(0)),
            cancellation: None,
            preparation_steps: 1,
            preparation_scratch_bytes: REQUEST_MEMORY + 1,
            target: None,
        })
        .unwrap()],
    );
    let partitions = std::collections::BTreeMap::new();
    let lease = checked_lease(REQUEST_MEMORY, 10_000, CancellationToken::new());
    let request = InvariantExecutionRequest::from_profile_with_contract(
        InvariantRequestProfile::CommitBoundary,
        &view,
        InvariantObservation::committed_branch(&partitions),
        version,
        None,
        None,
    );
    let result = InvariantEngine::from_view(&view)
        .execute(&worth_execution::ExecutionRequest::leased(&lease), request);
    assert_eq!(evaluation_visits.load(Ordering::SeqCst), 0);
    // The claim is charged to the lease before it is made, so the request's
    // own memory policy refuses it: the claim alone exceeds the request.
    let Err(crate::contracts::transactions::data::outcomes::transaction_commit_error::TransactionCommitError::Execution {
        denial:
            crate::contracts::transactions::data::outcomes::transaction_commit_error::CommitExecutionDenial {
                kind:
                    crate::contracts::transactions::data::outcomes::transaction_commit_error::CommitExecutionDenialKind::Cause(
                        crate::contracts::execution::denial_cause::RelationalExecutionDenialCause::PolicyMemoryExhausted {
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
