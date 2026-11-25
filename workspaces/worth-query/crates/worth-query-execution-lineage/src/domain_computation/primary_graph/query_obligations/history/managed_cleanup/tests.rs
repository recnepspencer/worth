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
        cold_release: None,
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
            retired_branches: Default::default(),
            demand_lifecycles: Default::default(),
            entries: BTreeMap::new(),
            journal: Some(journal),
            cold_adoption_summary: QueryColdAdoptionReadmissionSummary::default(),
        })),
        // This value test fabricates no publication/resource authority.
        medium: Default::default(),
        publication_driver: Mutex::new(()),
        basis_pins: std::sync::OnceLock::new(),
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
            retired_branches: Default::default(),
            demand_lifecycles: Default::default(),
            entries: BTreeMap::new(),
            journal: Some(QueryObligationJournalState::from_ledger(
                QueryObligationLedger::empty(home),
            )),
            cold_adoption_summary: QueryColdAdoptionReadmissionSummary::default(),
        })),
        medium: Default::default(),
        publication_driver: Mutex::new(()),
        basis_pins: std::sync::OnceLock::new(),
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

#[test]
fn retired_branch_demand_selection_isolates_recreated_same_name_incarnation() {
    use std::num::NonZeroU64;
    use worth_foundational::facade::{
        CanonicalDigestId, DurableCompositeCommitIdentity, DurableWorldOwnerIdentity,
        WorldOwnerSlotIdentity,
    };

    let home = HomeIdentity::from_observed_bytes([11; 16]).unwrap();
    let owner =
        DurableWorldOwnerIdentity::from_recorded(home, WorldOwnerSlotIdentity::from_recorded(1));
    let c1 = DurableCompositeCommitIdentity::from_recorded(owner, 1);
    let c2 = DurableCompositeCommitIdentity::from_recorded(owner, 2);
    let revision = CanonicalDigestId::new([3; 32]);
    let mut ledger = QueryObligationLedger::empty(home);
    let mut generations = QueryObligationPublicationGenerations::from_journal(&ledger);
    let install = ledger
        .prepare(
            QueryProgramSupportRecord::Install { revision },
            &mut generations,
        )
        .unwrap();
    ledger.readmit(install).unwrap();

    let make_demand =
        |pos: u64, basis: DurableCompositeCommitIdentity| QueryOutputDemandObligation {
            acquire_position: NonZeroU64::new(pos).unwrap(),
            revision,
            family: "first".to_owned(),
            producer: "producer".to_owned(),
            query: [5; 32],
            branch: "fork".to_owned(),
            basis,
            root: [7; 16],
            parameters: Vec::new(),
            scope: Vec::new(),
            performed_sources: Vec::new(),
            prerequisites: Vec::new(),
            checkpoint_prerequisites: Vec::new(),
        };
    let pos_l1 = NonZeroU64::new(2).unwrap();
    let pos_l2 = NonZeroU64::new(3).unwrap();
    let f1 = ledger
        .prepare(
            QueryOutputDemandRecord::Retain(make_demand(pos_l1.get(), c1)),
            &mut generations,
        )
        .unwrap();
    ledger.readmit(f1).unwrap();
    let f2 = ledger
        .prepare(
            QueryOutputDemandRecord::Retain(make_demand(pos_l2.get(), c2)),
            &mut generations,
        )
        .unwrap();
    ledger.readmit(f2).unwrap();

    let history = QueryObligationHistory {
        state: Arc::new(Mutex::new(QueryObligationState {
            pending_output_discharges: Default::default(),
            retired_branches: Default::default(),
            demand_lifecycles: BTreeMap::new(),
            entries: BTreeMap::new(),
            journal: Some(QueryObligationJournalState::from_ledger(ledger)),
            cold_adoption_summary: QueryColdAdoptionReadmissionSummary::default(),
        })),
        medium: Default::default(),
        basis_pins: std::sync::OnceLock::new(),
        publication_driver: Mutex::new(()),
    };
    // Both demands share the exact branch name "fork" across incarnations L1 and L2.
    history.record_output_demand_lifecycle(pos_l1, 1);
    history.record_output_demand_lifecycle(pos_l2, 2);

    // Retire L1 ("fork", 1) while L2 ("fork", 2) is live.
    history.mark_branch_retired("fork", 1);
    assert!(history.is_branch_retired("fork", 1));
    assert!(!history.is_branch_retired("fork", 2));
    assert_eq!(
        history.active_output_demands_for_branch("fork", 1),
        vec![pos_l1],
        "L1 cleanup must select only L1 demands, never re-created L2 demands on the same name"
    );
    assert_eq!(
        history.active_output_demands_for_branch("fork", 2),
        vec![pos_l2]
    );
    assert!(history.output_demand_retained(pos_l1));
    assert!(history.output_demand_retained(pos_l2));
}
