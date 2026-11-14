//! Canonical replay/drift must precede fresh independent-prefix planning.
use super::*;
pub(super) fn verify(host: &Host, original: &WorthQueryApplicationCommitReceipt) {
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    macro_rules! retry {
        ($value:expr) => {
            request
                .edit_envelope()
                .stage::<ProducedTree, _>(dependent::Intent { add: 7 })
                .stage::<ModelTree, _>(fixture::model::Intent {
                    key: "a".into(),
                    value: $value,
                    key_derived_target: false,
                })
                .stage::<ModelTree, _>(fixture::model::Intent {
                    key: "b".into(),
                    value: 4,
                    key_derived_target: false,
                })
                .idempotency("independent-sources-with-dependent".to_owned())
                .execute_performed::<Program>(host, ExecutionAllocationPolicy::SystemAllocation)
                .unwrap()
        };
    }
    let Outcome::RecoveredCommitted(recovered) = retry!(3) else {
        panic!("retry must return the authentic original receipt");
    };
    assert!(original.matches_recorded_original(&recovered));
    assert_eq!(original.commit_reference(), recovered.commit_reference());
    // The actual Model handler refuses this input before building a candidate.
    // Planning it first would lose precision and return the declaration cycle,
    // rather than the recorded whole-envelope intent drift.
    assert!(matches!(retry!(1001), Outcome::IdempotencyIntentDrift));
}
