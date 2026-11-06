//! Concurrent ordinary requests must converge on one original business result.
use super::{projection::HeadProjection, receipts, requests, OracleRuntime};

#[expect(
    clippy::disallowed_methods,
    reason = "The certification driver owns two concurrent external Host callers to prove same-key convergence; each uses the real execution request"
)]
pub fn execute(
    runtime: &OracleRuntime,
    key: u64,
    value: u64,
    before: &HeadProjection,
) -> requests::MutationOutcome {
    let start = std::sync::Barrier::new(2);
    let arrivals = std::thread::scope(|scope| {
        let attempt = || {
            start.wait();
            let outcome = requests::set(runtime, key, value);
            let projection = receipts::observe(&outcome, before);
            assert_eq!(
                projection.fate,
                receipts::MutationFate::Committed,
                "a concurrent same-key request resolves its original outcome: {outcome:?}"
            );
            projection
        };
        let first = scope.spawn(attempt);
        let second = scope.spawn(attempt);
        [first.join().unwrap(), second.join().unwrap()]
    });
    let original = requests::set(runtime, key, value);
    assert!(matches!(original, worth_query_host::facade::application_entry::WorthQueryApplicationMutationOutcome::AlreadyCommitted(_) | worth_query_host::facade::application_entry::WorthQueryApplicationMutationOutcome::RecoveredCommitted(_)),
        "the final same-key retry must read the original result: {original:?}");
    let expected = serde_json::to_value(receipts::observe(&original, before)).unwrap();
    for arrival in arrivals {
        assert_eq!(
            serde_json::to_value(arrival).unwrap(),
            expected,
            "both concurrent callers receive the same original durable meaning"
        );
    }
    original
}
