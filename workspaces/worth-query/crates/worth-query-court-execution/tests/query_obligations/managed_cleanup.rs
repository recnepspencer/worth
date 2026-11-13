//! Value-only journal rules; the directory court supplies actual release provenance.
use worth_foundational::facade::{HomeIdentity, ManagedCleanupObligationIdentity};
use worth_query_execution::facade::{
    domain_computation::{
        WorthQueryManagedRunCleanupDisposition, WorthQueryManagedRunCleanupFailureKind,
        WorthQueryManagedRunTerminalKind, WorthQueryMutationCleanupFailureKind,
    },
    query_obligations::*,
};

fn retain(
    failure: WorthQueryMutationCleanupFailureKind,
) -> (
    QueryObligationLedger,
    QueryObligationPublicationGenerations,
    ManagedCleanupObligationIdentity,
) {
    let home = HomeIdentity::from_observed_bytes([7; 16]).unwrap();
    let identity = ManagedCleanupObligationIdentity::from_recorded(home, std::num::NonZeroU64::MIN);
    let mut ledger = QueryObligationLedger::empty(home);
    let mut generations = QueryObligationPublicationGenerations::from_journal(&ledger);
    let frame = ledger
        .prepare(
            QueryManagedCleanupRecord::Retain(QueryManagedCleanupObligation {
                identity,
                run: "recorded-run".to_owned(),
                failure,
                original: None,
                release: None,
            }),
            &mut generations,
        )
        .unwrap();
    ledger
        .readmit(QueryObligationFrame::readmit(frame.bytes()).unwrap())
        .unwrap();
    (ledger, generations, identity)
}

#[test]
fn managed_cleanup_without_recorded_release_cannot_discharge_from_a_cold_fold() {
    let (mut ledger, mut generations, identity) =
        retain(WorthQueryMutationCleanupFailureKind::Native(
            WorthQueryManagedRunCleanupFailureKind::ProviderMemoryRetained,
        ));
    assert_eq!(
        ledger.prepare(
            QueryManagedCleanupRecord::Discharge { identity },
            &mut generations
        ),
        Err(QueryObligationDenial::CleanupReleaseUnrecorded { identity })
    );
    assert!(ledger.managed_cleanup().next().unwrap().release.is_none());
    let frame = ledger
        .prepare(
            QueryManagedCleanupRecord::Released {
                identity,
                release: QueryManagedCleanupRelease {
                    terminal: WorthQueryManagedRunTerminalKind::Failed,
                    disposition: WorthQueryManagedRunCleanupDisposition::RecoveryRequired,
                },
            },
            &mut generations,
        )
        .unwrap();
    ledger
        .readmit(QueryObligationFrame::readmit(frame.bytes()).unwrap())
        .unwrap();
    let row = ledger.managed_cleanup().next().unwrap();
    assert_eq!(row.identity, identity);
    assert_eq!(
        row.release.unwrap().terminal,
        WorthQueryManagedRunTerminalKind::Failed
    );
    assert_eq!(
        row.release.unwrap().disposition,
        WorthQueryManagedRunCleanupDisposition::RecoveryRequired
    );
    let frame = ledger
        .prepare(
            QueryManagedCleanupRecord::Discharge { identity },
            &mut generations,
        )
        .unwrap();
    ledger
        .readmit(QueryObligationFrame::readmit(frame.bytes()).unwrap())
        .unwrap();
    assert!(ledger.managed_cleanup().next().is_none());
    assert_eq!(
        ledger.prepare(
            QueryManagedCleanupRecord::Discharge { identity },
            &mut generations
        ),
        Err(QueryObligationDenial::UnknownObligation)
    );
}

#[test]
fn managed_cleanup_inconsistencies_remain_distinct_and_cannot_be_released() {
    for failure in [
        WorthQueryMutationCleanupFailureKind::AffinityInconsistent,
        WorthQueryMutationCleanupFailureKind::CompletionInconsistent,
    ] {
        let (ledger, mut generations, identity) = retain(failure);
        assert_eq!(ledger.managed_cleanup().next().unwrap().failure, failure);
        assert_eq!(
            ledger.prepare(
                QueryManagedCleanupRecord::Released {
                    identity,
                    release: QueryManagedCleanupRelease {
                        terminal: WorthQueryManagedRunTerminalKind::Failed,
                        disposition: WorthQueryManagedRunCleanupDisposition::CleanupComplete,
                    }
                },
                &mut generations
            ),
            Err(QueryObligationDenial::CleanupInconsistent { identity })
        );
        assert_eq!(
            ledger.prepare(
                QueryManagedCleanupRecord::Discharge { identity },
                &mut generations
            ),
            Err(QueryObligationDenial::CleanupInconsistent { identity })
        );
    }
}

#[test]
fn managed_cleanup_preserves_distinct_world_subject_identity_families() {
    use worth_foundational::facade::{
        DurableCompositePublicationAttemptIdentity, DurableProductUnpublishedOwnerEffectsIdentity,
        DurableWorldOwnerIdentity, WorldOwnerSlotIdentity,
    };
    let home = HomeIdentity::from_observed_bytes([9; 16]).unwrap();
    let owner =
        DurableWorldOwnerIdentity::from_recorded(home, WorldOwnerSlotIdentity::from_recorded(0));
    for original in [
        QueryManagedCleanupOriginal::Attempt(
            DurableCompositePublicationAttemptIdentity::from_recorded(owner, 17),
        ),
        QueryManagedCleanupOriginal::Unpublished(
            DurableProductUnpublishedOwnerEffectsIdentity::from_recorded(owner, 17),
        ),
    ] {
        let mut ledger = QueryObligationLedger::empty(home);
        let mut generations = QueryObligationPublicationGenerations::from_journal(&ledger);
        let identity =
            ManagedCleanupObligationIdentity::from_recorded(home, std::num::NonZeroU64::MIN);
        let frame = ledger
            .prepare(
                QueryManagedCleanupRecord::Retain(QueryManagedCleanupObligation {
                    identity,
                    run: "original-world-subject".to_owned(),
                    failure: WorthQueryMutationCleanupFailureKind::Native(
                        WorthQueryManagedRunCleanupFailureKind::ProviderMemoryRetained,
                    ),
                    original: Some(original),
                    release: None,
                }),
                &mut generations,
            )
            .unwrap();
        ledger
            .readmit(QueryObligationFrame::readmit(frame.bytes()).unwrap())
            .unwrap();
        assert_eq!(
            ledger.managed_cleanup().next().unwrap().original,
            Some(original)
        );
    }
}
