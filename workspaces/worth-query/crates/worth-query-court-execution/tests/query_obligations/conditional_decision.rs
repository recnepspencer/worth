//! Value-only protocol evidence; the Host court supplies real cold continuation.
use worth_foundational::facade::{HomeIdentity, ManagedCleanupObligationIdentity};
use worth_query_execution::facade::{
    domain_computation::{
        WorthQueryManagedRunCleanupDisposition, WorthQueryManagedRunCleanupFailureKind,
        WorthQueryManagedRunTerminalKind, WorthQueryMutationCleanupFailureKind,
    },
    query_obligations::*,
};

fn append(ledger: &mut QueryObligationLedger, record: impl Into<QueryObligationRecord>) {
    let mut generations = QueryObligationPublicationGenerations::from_journal(ledger);
    let frame = ledger.prepare(record, &mut generations).unwrap();
    let decoded = QueryObligationFrame::readmit(frame.bytes()).unwrap();
    assert_eq!(decoded.record(), frame.record());
    ledger.readmit(decoded).unwrap();
}

#[test]
fn conditional_temporal_custody_keeps_keys_and_requires_logical_cleanup_discharge() {
    let home = HomeIdentity::from_observed_bytes([3; 16]).unwrap();
    let identity = ManagedCleanupObligationIdentity::from_recorded(home, std::num::NonZeroU64::MIN);
    let mut ledger = QueryObligationLedger::empty(home);
    append(
        &mut ledger,
        QueryManagedCleanupRecord::Retain(QueryManagedCleanupObligation {
            identity,
            run: "recorded-cleanup".to_owned(),
            failure: WorthQueryMutationCleanupFailureKind::Native(
                WorthQueryManagedRunCleanupFailureKind::ProviderMemoryRetained,
            ),
            original: None,
            release: None,
        }),
    );
    for (ordinal, posture) in [
        QueryConditionalPosture::CleanupRequired(identity),
        QueryConditionalPosture::Retryable,
        QueryConditionalPosture::CommitRetryable,
        QueryConditionalPosture::ProductStale,
    ]
    .into_iter()
    .enumerate()
    {
        let row = QueryConditionalDecision {
            key: [ordinal as u8; 32],
            intent: [9; 32],
            posture,
        };
        append(
            &mut ledger,
            QueryConditionalDecisionRecord::Retain(row.clone()),
        );
        let mut generations = QueryObligationPublicationGenerations::from_journal(&ledger);
        assert_eq!(
            ledger.prepare(
                QueryConditionalDecisionRecord::Finish {
                    key: row.key,
                    intent: [8; 32]
                },
                &mut generations
            ),
            Err(QueryObligationDenial::ObligationSubjectMismatch)
        );
        if ordinal == 0 {
            assert_eq!(
                ledger.prepare(
                    QueryConditionalDecisionRecord::Finish {
                        key: row.key,
                        intent: row.intent
                    },
                    &mut generations
                ),
                Err(QueryObligationDenial::ConditionalCleanupPending {
                    key: row.key,
                    identity
                })
            );
        }
        assert!(ledger.conditional_decisions().any(|known| known == &row));
    }
    append(
        &mut ledger,
        QueryManagedCleanupRecord::Released {
            identity,
            release: QueryManagedCleanupRelease {
                terminal: WorthQueryManagedRunTerminalKind::Failed,
                disposition: WorthQueryManagedRunCleanupDisposition::RecoveryRequired,
            },
        },
    );
    // Actual process release alone does not finish the logical cleanup.
    let mut generations = QueryObligationPublicationGenerations::from_journal(&ledger);
    assert_eq!(
        ledger.prepare(
            QueryConditionalDecisionRecord::Finish {
                key: [0; 32],
                intent: [9; 32]
            },
            &mut generations
        ),
        Err(QueryObligationDenial::ConditionalCleanupPending {
            key: [0; 32],
            identity
        })
    );
    append(
        &mut ledger,
        QueryManagedCleanupRecord::Discharge { identity },
    );
    for ordinal in 0..4 {
        append(
            &mut ledger,
            QueryConditionalDecisionRecord::Finish {
                key: [ordinal; 32],
                intent: [9; 32],
            },
        );
    }
    assert_eq!(ledger.conditional_decisions().count(), 0);
}
