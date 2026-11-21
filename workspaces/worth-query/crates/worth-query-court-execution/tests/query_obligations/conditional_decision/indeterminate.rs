//! R9 `OperationIndeterminate` rows: retained across fold, joined only to their
//! own recovered World original, and finished exactly once in each state.
use super::append;
use worth_foundational::facade::{
    DurableCompositePublicationAttemptIdentity, DurableWorldOwnerIdentity, HomeIdentity,
    WorldOwnerSlotIdentity,
};
use worth_query_execution::facade::query_obligations::*;

fn attempt(home: HomeIdentity, ordinal: u64) -> DurableCompositePublicationAttemptIdentity {
    DurableCompositePublicationAttemptIdentity::from_recorded(
        DurableWorldOwnerIdentity::from_recorded(home, WorldOwnerSlotIdentity::from_recorded(1)),
        ordinal,
    )
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

fn row(key: u8, posture: QueryConditionalPosture) -> QueryConditionalDecision {
    QueryConditionalDecision {
        key: [key; 32],
        intent: [9; 32],
        posture,
    }
}

/// Each cold state ends the same retained row exactly once: published and
/// no-effect rows finish unclassified; a recovered row finishes after joining
/// its own original (the retry continued it).
#[test]
fn an_indeterminate_row_is_retained_joined_to_its_own_original_and_finished_once() {
    use QueryConditionalOriginal::{Recovered, Unclassified};
    use QueryConditionalPosture::Indeterminate;
    let home = HomeIdentity::from_observed_bytes([4; 16]).unwrap();
    let foreign = HomeIdentity::from_observed_bytes([5; 16]).unwrap();
    for (key, state, joined) in [
        (1, "published, finished with the original occurrence", None),
        (2, "no committed open, released as no-effect", None),
        (
            3,
            "recovered and unpublished, joined then continued",
            Some(attempt(home, 7)),
        ),
    ] {
        let mut ledger = QueryObligationLedger::empty(home);
        let recorded = row(key, Indeterminate(Unclassified));
        append(
            &mut ledger,
            QueryConditionalDecisionRecord::Retain(recorded.clone()),
        );
        assert_eq!(
            ledger.conditional_decisions().collect::<Vec<_>>(),
            [&recorded],
            "{state}: the unclassified row survives encode, decode and fold"
        );
        assert_eq!(
            refused(
                &ledger,
                QueryConditionalDecisionRecord::Retain(QueryConditionalDecision {
                    intent: [8; 32],
                    ..recorded.clone()
                })
            ),
            QueryObligationDenial::ObligationSubjectMismatch,
            "{state}: another intent cannot take the original key"
        );
        assert_eq!(
            refused(
                &ledger,
                QueryConditionalDecisionRecord::Retain(row(
                    key,
                    Indeterminate(Recovered(attempt(foreign, 7)))
                ))
            ),
            QueryObligationDenial::ForeignHome,
            "{state}: a foreign Home's original never joins"
        );
        if let Some(original) = joined {
            let joined_row = row(key, Indeterminate(Recovered(original)));
            append(
                &mut ledger,
                QueryConditionalDecisionRecord::Retain(joined_row.clone()),
            );
            assert_eq!(
                ledger.conditional_decisions().collect::<Vec<_>>(),
                [&joined_row]
            );
            for elsewhere in [
                Indeterminate(Recovered(attempt(home, 8))),
                Indeterminate(Unclassified),
                QueryConditionalPosture::Retryable,
            ] {
                assert_eq!(
                    refused(
                        &ledger,
                        QueryConditionalDecisionRecord::Retain(row(key, elsewhere))
                    ),
                    QueryObligationDenial::ObligationSubjectMismatch,
                    "{state}: a joined row keeps exactly its own original ({elsewhere:?})"
                );
            }
        }
        assert_eq!(
            refused(
                &ledger,
                QueryConditionalDecisionRecord::Finish {
                    key: [key; 32],
                    intent: [8; 32]
                }
            ),
            QueryObligationDenial::ObligationSubjectMismatch,
            "{state}: a mismatched intent cannot finish the row"
        );
        let finish = QueryConditionalDecisionRecord::Finish {
            key: [key; 32],
            intent: [9; 32],
        };
        append(&mut ledger, finish.clone());
        assert_eq!(ledger.conditional_decisions().count(), 0, "{state}");
        assert_eq!(
            refused(&ledger, finish),
            QueryObligationDenial::UnknownObligation,
            "{state}: the row finishes exactly once"
        );
    }
}
