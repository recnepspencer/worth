use super::publication::{prepare_relational, ready_relational_publication, setup};
use crate::lifecycle::RuntimeWorldObservationService;
use crate::publication::{CompositeLateCancellationPosture, RuntimeWorldPublicationOutcome};

#[test]
fn performed_publication_page_resumes_and_protects_canonical_owner_evidence() {
    use std::num::NonZeroUsize;

    let (fixture, owner, expected) = setup();
    let cell = owner.state.branches.root_cell().expect("bootstrapped cell");
    let ready = ready_relational_publication(&fixture, &owner, expected.clone());
    let performed = match ready.publish(&cell, CompositeLateCancellationPosture::NotRequested) {
        RuntimeWorldPublicationOutcome::Performed(performed) => performed,
        other => panic!("publication performs: {other:?}"),
    };
    let one = NonZeroUsize::new(1).unwrap();
    let root_page = owner
        .state
        .history
        .performed_publication_page(None, one)
        .unwrap();
    assert_eq!(root_page.examined(), 1);
    assert!(root_page.rows().is_empty());
    let cursor = root_page
        .next_after()
        .expect("ordinary commit follows root")
        .clone();
    let before = owner
        .state
        .history
        .counters()
        .direct_protection_acquisitions();
    let publication_page = owner
        .state
        .history
        .performed_publication_page(Some(&cursor), one)
        .unwrap();
    assert_eq!(publication_page.examined(), 1);
    assert!(publication_page.next_after().is_none());
    assert!(!publication_page.pending());
    assert!(owner
        .state
        .history
        .publication_frontier_is_current(publication_page.frontier())
        .unwrap());
    let row = &publication_page.rows()[0];
    assert_eq!(row.commit().identity(), performed.commit().identity());
    assert_eq!(
        row.product_incarnation(),
        expected.snapshot().lifecycle_incarnation()
    );
    assert_eq!(row.product_branch(), expected.branch_identity());
    assert_eq!(
        row.component_results().relational_publication_identity(),
        performed
            .commit()
            .relational_publication_identity()
            .cloned()
    );
    assert_eq!(
        owner
            .state
            .history
            .counters()
            .direct_protection_acquisitions(),
        before + 1
    );
    let releases = owner.state.history.counters().direct_protection_releases();
    drop(publication_page);
    assert_eq!(
        owner.state.history.counters().direct_protection_releases(),
        releases + 1
    );
}

#[test]
fn performed_publication_lease_blocks_explicit_reclamation_after_branch_retirement() {
    use crate::history::CompositeHistoryReclamationRequest;
    use crate::lifecycle::RuntimeWorldBranchService;

    let (fixture, owner, expected) = setup();
    let cell = owner.state.branches.root_cell().expect("bootstrapped cell");
    let ready = ready_relational_publication(&fixture, &owner, expected.clone());
    let performed = match ready.publish(&cell, CompositeLateCancellationPosture::NotRequested) {
        RuntimeWorldPublicationOutcome::Performed(performed) => performed,
        other => panic!("publication performs: {other:?}"),
    };
    let identity = performed.commit().identity().clone();
    let lease = owner
        .state
        .history
        .protect_performed_publication(&identity)
        .unwrap();
    assert_eq!(lease.commit_identity(), &identity);
    let mut consumed = performed.consume();
    drop(consumed.take_successor_observation());
    drop(consumed);
    let current = RuntimeWorldObservationService::observe_product_branch(
        owner.as_ref(),
        expected.branch_identity(),
    )
    .unwrap();
    let _retirement =
        RuntimeWorldBranchService::retire_product_branch(owner.as_ref(), &current).unwrap();
    drop(current);
    let request = || {
        CompositeHistoryReclamationRequest::new(owner.owner_identity(), vec![identity.clone()], 1)
    };
    assert_eq!(
        owner
            .state
            .history
            .reclaim_batch(request())
            .unwrap()
            .skipped_protected(),
        1
    );
    drop(lease);
    assert_eq!(
        owner
            .state
            .history
            .reclaim_batch(request())
            .unwrap()
            .reclaimed_commits(),
        &[identity]
    );
}

#[test]
fn publication_frontier_rejects_later_history_reservation() {
    use std::num::NonZeroUsize;

    let (fixture, owner, expected) = setup();
    let page = owner
        .state
        .history
        .performed_publication_page(None, NonZeroUsize::new(1).unwrap())
        .unwrap();
    assert!(owner
        .state
        .history
        .publication_frontier_is_current(page.frontier())
        .unwrap());
    let _prepared = prepare_relational(&fixture, &owner, expected, "frontier");
    assert!(!owner
        .state
        .history
        .publication_frontier_is_current(page.frontier())
        .unwrap());
}
