//! A real proposal cannot fund each native/custom dispatch from a reset allowance.
use super::*;

#[test]
fn the_original_proposal_account_covers_both_stages_and_their_handoff() {
    let (fixture, hooks) = fixture(16);
    let before = fixture.head();
    let funded =
        super::super::fixture::request(8 * 1024 * 1024, 4_000_000, CancellationToken::new(), None);
    let (candidate, charges) = crate::execution::work_observation::capture(|| {
        fixture.validate(&ExecutionRequest::leased(&funded), fixture.chain_batch(2))
    });
    let candidate = candidate.unwrap();
    assert!(
        candidate.commit_work_budget.is_some(),
        "the real validation owner carries its account"
    );
    assert_eq!(candidate.commit_boundary.results().len(), 2);
    assert_eq!(candidate.commit_boundary.decision_log().len(), 2);
    assert_eq!(hooks.observed(), (1, 1));
    drop(candidate);
    assert_eq!(funded.charged_memory_bytes(), 0);
    assert!(
        charges.len() > 3,
        "the proposal pays multiple real dependent dispatches"
    );
    let largest = *charges.iter().max().unwrap();
    assert!(largest > 0);
    assert!(charges.iter().sum::<u64>() > largest);
    // Each isolated dispatch fitted this bound on the completing run. Resetting
    // the allowance at a native/custom or validation/preparation handoff is not
    // allowed to turn it into a completing whole-proposal budget.
    let constrained =
        super::super::fixture::request(8 * 1024 * 1024, largest, CancellationToken::new(), None);
    let error = fixture
        .validate(
            &ExecutionRequest::leased(&constrained),
            fixture.chain_batch(2),
        )
        .unwrap_err();
    let TransactionCommitError::Execution { denial, .. } = error else {
        panic!("a cumulative work stop must retain its original cause: {error:?}");
    };
    assert_eq!(
        denial.kind,
        crate::transactions::data::CommitExecutionDenialKind::Cause(
            crate::execution::RelationalExecutionDenialCause::WorkExhausted,
        )
    );
    assert_eq!(fixture.head(), before);
    assert_eq!(constrained.charged_memory_bytes(), 0);
    let baseline = hooks.observed();
    let receipt = fixture
        .publish(&ExecutionRequest::leased(&funded), fixture.chain_batch(2))
        .expect("same proposal with the original funded policy publishes");
    assert_eq!(hooks.observed(), (baseline.0 + 1, baseline.1 + 1));
    assert_ne!(fixture.head(), before);
    crate::tests::support::release_test_commit_snapshot(&fixture.runtime, &receipt);
    assert_eq!(funded.charged_memory_bytes(), 0);
}
