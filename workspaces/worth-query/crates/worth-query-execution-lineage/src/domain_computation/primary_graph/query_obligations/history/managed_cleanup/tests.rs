//! Retention acknowledgment is independent of a later publication's fate.
use super::*;
use worth_foundational::facade::HomeIdentity;

#[test]
fn managed_cleanup_acknowledgment_survives_a_later_release_no_effect() {
    let home = HomeIdentity::from_observed_bytes([7; 16]).unwrap();
    let identity = ManagedCleanupObligationIdentity::from_recorded(home, std::num::NonZeroU64::MIN);
    let mut ledger = QueryObligationLedger::empty(home);
    let mut generations = QueryObligationPublicationGenerations::from_journal(&ledger);
    let frame = ledger.prepare(QueryManagedCleanupRecord::Retain(QueryManagedCleanupObligation {
        identity,
        run: "original-run".to_owned(),
        failure: crate::domain_computation::WorthQueryMutationCleanupFailureKind::Native(
            crate::domain_computation::WorthQueryManagedRunCleanupFailureKind::ProviderMemoryRetained),
        original: None,
        release: None,
    }), &mut generations).unwrap();
    ledger.readmit(frame).unwrap();
    let mut journal = QueryObligationJournalState::from_ledger(ledger);
    journal.prepare_pending(QueryManagedCleanupRecord::Released {
        identity,
        release: QueryManagedCleanupRelease {
            terminal: crate::domain_computation::WorthQueryManagedRunTerminalKind::Failed,
            disposition: crate::domain_computation::WorthQueryManagedRunCleanupDisposition::CleanupComplete,
        },
    }).unwrap();
    let history = Arc::new(QueryObligationHistory {
        state: Arc::new(Mutex::new(QueryObligationState {
            pending_output_discharges: Default::default(),
            entries: BTreeMap::new(),
            journal: Some(journal),
        })),
        // This value test fabricates no publication/resource authority.
        medium: Default::default(),
        publication_driver: Mutex::new(()),
    });
    let mut publication = ManagedCleanupPublication {
        history: Arc::clone(&history),
        identity,
        pending_denial: None,
    };
    // An unsettled later release cannot lose its original retention owner.
    assert_eq!(
        publication.admit_retention_settlement(Err(QueryObligationDenial::PublicationPending)),
        Err(QueryObligationDenial::PublicationPending)
    );
    // Model the shared driver's no-effect cutover. No physical no-effect or
    // resource-release claim is made by this data-only fixture.
    history.state().journal.as_mut().unwrap().pending = None;
    publication
        .admit_retention_settlement(Err(QueryObligationDenial::PublicationNoEffect))
        .unwrap();
    assert_eq!(publication.identity(), identity);
    assert!(publication.pending_denial.is_none());
    let rows = history.managed_cleanup_obligations();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].identity, identity);
    assert!(rows[0].release.is_none());
    unacknowledged_retention_no_effect_refuses_acknowledgment();
}

fn unacknowledged_retention_no_effect_refuses_acknowledgment() {
    let home = HomeIdentity::from_observed_bytes([8; 16]).unwrap();
    let identity = ManagedCleanupObligationIdentity::from_recorded(home, std::num::NonZeroU64::MIN);
    let history = Arc::new(QueryObligationHistory {
        state: Arc::new(Mutex::new(QueryObligationState {
            pending_output_discharges: Default::default(),
            entries: BTreeMap::new(),
            journal: Some(QueryObligationJournalState::from_ledger(
                QueryObligationLedger::empty(home),
            )),
        })),
        medium: Default::default(),
        publication_driver: Mutex::new(()),
    });
    let mut publication = ManagedCleanupPublication {
        history,
        identity,
        pending_denial: Some(QueryObligationDenial::PublicationUnresolved),
    };
    assert_eq!(
        publication.admit_retention_settlement(Err(QueryObligationDenial::PublicationNoEffect)),
        Err(QueryObligationDenial::PublicationNoEffect)
    );
    assert_eq!(
        publication.admit_retention_settlement(Ok(())),
        Err(QueryObligationDenial::UnknownObligation)
    );
    assert_eq!(
        publication.pending_denial,
        Some(QueryObligationDenial::PublicationUnresolved)
    );
}
