//! R9 `OperationIndeterminate` fold and codec laws: durable product affinity
//! survives round-trip, unclassified rows never downgrade to retry postures,
//! foreign Homes and wrong subjects are refused, a joined original stays fixed,
//! and a row finishes exactly once. World-driven classification of the three
//! fates is proved at the real World boundary in the Host process courts.
use super::append;
use worth_foundational::facade::{
    DurableCompositePublicationAttemptIdentity, DurableProductBranchIncarnation,
    DurableWorldOwnerIdentity, HomeIdentity, ManagedCleanupObligationIdentity,
    WorldOwnerSlotIdentity,
};
use worth_query_execution::facade::{
    domain_computation::{
        WorthQueryManagedRunCleanupFailureKind, WorthQueryMutationCleanupFailureKind,
    },
    query_obligations::*,
};

fn owner(home: HomeIdentity, slot: u64) -> DurableWorldOwnerIdentity {
    DurableWorldOwnerIdentity::from_recorded(home, WorldOwnerSlotIdentity::from_recorded(slot))
}

fn product(home: HomeIdentity, slot: u64, ordinal: u64) -> DurableProductBranchIncarnation {
    DurableProductBranchIncarnation::from_recorded(owner(home, slot), ordinal)
}

fn attempt(
    home: HomeIdentity,
    slot: u64,
    ordinal: u64,
) -> DurableCompositePublicationAttemptIdentity {
    DurableCompositePublicationAttemptIdentity::from_recorded(owner(home, slot), ordinal)
}

fn refused(
    ledger: &QueryObligationLedger,
    record: impl Into<QueryObligationRecord>,
) -> QueryObligationDenial {
    let mut generations = QueryObligationPublicationGenerations::from_journal(ledger);
    ledger
        .prepare(record, &mut generations)
        .expect_err("the fold refuses this record")
}

#[test]
fn indeterminate_obligation_preserves_product_affinity_uncertainty_and_single_finish() {
    use QueryConditionalOriginal::{Recovered, Unclassified};
    use QueryConditionalPosture::{
        CleanupRequired, CommitRetryable, Indeterminate, ProductStale, Retryable,
    };
    let home = HomeIdentity::from_observed_bytes([4; 16]).unwrap();
    let foreign = HomeIdentity::from_observed_bytes([5; 16]).unwrap();
    let branch = product(home, 1, 2);
    let mut ledger = QueryObligationLedger::empty(home);
    let cleanup = ManagedCleanupObligationIdentity::from_recorded(home, std::num::NonZeroU64::MIN);
    append(
        &mut ledger,
        QueryManagedCleanupRecord::Retain(QueryManagedCleanupObligation {
            identity: cleanup,
            run: "recorded-cleanup".to_owned(),
            failure: WorthQueryMutationCleanupFailureKind::Native(
                WorthQueryManagedRunCleanupFailureKind::ProviderMemoryRetained,
            ),
            original: None,
            release: None,
            cold_release: None,
        }),
    );

    let row = |posture| QueryConditionalDecision {
        key: [1; 32],
        intent: [9; 32],
        product: branch,
        posture,
    };
    // Foreign product affinity is refused before any row is retained.
    assert_eq!(
        refused(
            &ledger,
            QueryConditionalDecisionRecord::Retain(QueryConditionalDecision {
                product: product(foreign, 1, 2),
                ..row(Indeterminate(Unclassified))
            })
        ),
        QueryObligationDenial::ForeignHome,
    );

    // Unclassified indeterminate row survives encode, decode, and fold with
    // its exact durable product branch incarnation.
    let unclassified = row(Indeterminate(Unclassified));
    append(
        &mut ledger,
        QueryConditionalDecisionRecord::Retain(unclassified.clone()),
    );
    assert_eq!(
        ledger.conditional_decisions().collect::<Vec<_>>(),
        [&unclassified],
    );

    // Neither another intent nor another product branch incarnation may take
    // the key, and no ordinary retry posture may erase unclassified uncertainty.
    assert_eq!(
        refused(
            &ledger,
            QueryConditionalDecisionRecord::Retain(QueryConditionalDecision {
                intent: [8; 32],
                ..unclassified.clone()
            })
        ),
        QueryObligationDenial::ObligationSubjectMismatch,
    );
    assert_eq!(
        refused(
            &ledger,
            QueryConditionalDecisionRecord::Retain(QueryConditionalDecision {
                product: product(home, 1, 3),
                ..unclassified.clone()
            })
        ),
        QueryObligationDenial::ObligationSubjectMismatch,
    );
    for downgrade in [
        Retryable,
        CommitRetryable,
        ProductStale,
        CleanupRequired(cleanup),
    ] {
        assert_eq!(
            refused(
                &ledger,
                QueryConditionalDecisionRecord::Retain(row(downgrade))
            ),
            QueryObligationDenial::ObligationSubjectMismatch,
            "unclassified uncertainty never downgrades to {downgrade:?}"
        );
    }

    // Foreign Home or mismatched World slot cannot join the row.
    assert_eq!(
        refused(
            &ledger,
            QueryConditionalDecisionRecord::Retain(row(Indeterminate(Recovered(attempt(
                foreign, 1, 7
            )))))
        ),
        QueryObligationDenial::ForeignHome,
    );
    assert_eq!(
        refused(
            &ledger,
            QueryConditionalDecisionRecord::Retain(row(Indeterminate(Recovered(attempt(
                home, 2, 7
            )))))
        ),
        QueryObligationDenial::ObligationSubjectMismatch,
    );

    // Joining the row's own original survives round-trip and stays fixed.
    let joined = row(Indeterminate(Recovered(attempt(home, 1, 7))));
    append(
        &mut ledger,
        QueryConditionalDecisionRecord::Retain(joined.clone()),
    );
    assert_eq!(
        ledger.conditional_decisions().collect::<Vec<_>>(),
        [&joined],
    );
    for elsewhere in [
        Indeterminate(Recovered(attempt(home, 1, 8))),
        Indeterminate(Unclassified),
        Retryable,
    ] {
        assert_eq!(
            refused(
                &ledger,
                QueryConditionalDecisionRecord::Retain(row(elsewhere))
            ),
            QueryObligationDenial::ObligationSubjectMismatch,
            "a joined row keeps exactly its own original ({elsewhere:?})"
        );
    }

    // Wrong intent cannot finish the row; matching intent finishes it once.
    assert_eq!(
        refused(
            &ledger,
            QueryConditionalDecisionRecord::Finish {
                key: [1; 32],
                intent: [8; 32],
            }
        ),
        QueryObligationDenial::ObligationSubjectMismatch,
    );
    let finish = QueryConditionalDecisionRecord::Finish {
        key: [1; 32],
        intent: [9; 32],
    };
    append(&mut ledger, finish.clone());
    assert_eq!(ledger.conditional_decisions().count(), 0);
    assert_eq!(
        refused(&ledger, finish),
        QueryObligationDenial::UnknownObligation,
    );
}
