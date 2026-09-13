use std::sync::{Arc, Barrier};
use std::thread;

use super::{ProductBranchObservation, ProductBranchObservationAuthorityDenial};
use crate::branch::reference_cell::{
    ProductBranchReferenceCell, ProductBranchReferenceObservationFailure,
};
use crate::branch::reference_test_fixture as fixture;
use crate::branch::ProductBranchReferenceSnapshot;
use crate::history::{CompositeHistoryCatalog, CompositeHistoryReclamationRequest};

fn product_cell(
    snapshot: ProductBranchReferenceSnapshot,
    catalog: &CompositeHistoryCatalog,
    fixture: &fixture::RealReferenceFixture,
) -> ProductBranchReferenceCell {
    let protection = fixture::product_head_protection(fixture, catalog, snapshot);
    ProductBranchReferenceCell::new(protection).expect("protected cell admission")
}

#[test]
fn observation_binding_returns_both_exact_tokens_on_each_mismatch() {
    let (mut fixture, catalog, root) = fixture::installed_root();
    let first = fixture::install_ordinary(&mut fixture, &catalog, root.as_ref());
    let second = fixture::install_ordinary(&mut fixture, &catalog, root.as_ref());
    let snapshot = fixture::initial_snapshot(&mut fixture, Arc::clone(&first));

    let before_history = catalog.counters();
    let component_mismatch = ProductBranchObservation::owner_issued(
        snapshot.clone(),
        fixture
            .owner
            .issue_observation(second.as_ref())
            .expect("real second component retention"),
        catalog
            .protect_explicit_commit(first.as_ref())
            .expect("real first history protection"),
    )
    .expect_err("component retention must bind to the selected commit");
    assert_eq!(
        component_mismatch.denial(),
        ProductBranchObservationAuthorityDenial::ComponentCommitOrBasisMismatch
    );
    let (_, components, history) = component_mismatch.into_parts();
    drop(components);
    drop(history);
    assert_eq!(fixture.owner.active_component_obligation_count(), 6);
    assert_eq!(
        catalog.counters().direct_protection_releases(),
        before_history.direct_protection_releases() + 1
    );

    let before_history = catalog.counters();
    let history_mismatch = ProductBranchObservation::owner_issued(
        snapshot,
        fixture
            .owner
            .issue_observation(first.as_ref())
            .expect("real first component retention"),
        catalog
            .protect_explicit_commit(second.as_ref())
            .expect("real second history protection"),
    )
    .expect_err("history protection must bind to the selected commit");
    assert_eq!(
        history_mismatch.denial(),
        ProductBranchObservationAuthorityDenial::HistoryCommitOrOwnerMismatch
    );
    let (_, components, history) = history_mismatch.into_parts();
    drop(components);
    drop(history);
    assert_eq!(fixture.owner.active_component_obligation_count(), 6);
    assert_eq!(
        catalog.counters().direct_protection_releases(),
        before_history.direct_protection_releases() + 1
    );
}

#[test]
fn observation_denials_issue_no_managed_authority_or_stranded_token() {
    let (mut fixture, catalog, root) = fixture::installed_root();
    let snapshot = fixture::initial_snapshot(&mut fixture, Arc::clone(&root));
    let cell = product_cell(snapshot, &catalog, &fixture);
    let (foreign_fixture, foreign_catalog, _) = fixture::installed_root();

    let before_owner = fixture.owner.cost_snapshot();
    let before_history = catalog.counters();
    assert!(matches!(
        cell.observe(&foreign_catalog, &fixture.owner),
        Err(ProductBranchReferenceObservationFailure::HistoryProtection(
            _
        ))
    ));
    assert_eq!(fixture.owner.cost_snapshot(), before_owner);
    assert_eq!(fixture.owner.active_component_obligation_count(), 4);
    assert_eq!(catalog.counters(), before_history);

    let before_foreign_owner = foreign_fixture.owner.cost_snapshot();
    let before_history = catalog.counters();
    assert!(matches!(
        cell.observe(&catalog, &foreign_fixture.owner),
        Err(ProductBranchReferenceObservationFailure::Retention(_))
    ));
    let after_foreign_owner = foreign_fixture.owner.cost_snapshot();
    assert_eq!(
        after_foreign_owner.batch_denied(),
        before_foreign_owner.batch_denied() + 1
    );
    assert_eq!(
        after_foreign_owner.relational_contacts(),
        before_foreign_owner.relational_contacts()
    );
    assert_eq!(
        after_foreign_owner.signal_contacts(),
        before_foreign_owner.signal_contacts()
    );
    assert_eq!(foreign_fixture.owner.unique_pin_count(), 2);
    assert_eq!(foreign_fixture.owner.active_component_obligation_count(), 2);
    assert_eq!(
        catalog.counters().direct_protection_releases(),
        before_history.direct_protection_releases() + 1
    );
}

#[test]
fn cloned_observations_share_authority_and_release_final_sibling_head() {
    let (mut fixture, catalog, root) = fixture::installed_root();
    let selected = fixture::install_ordinary(&mut fixture, &catalog, root.as_ref());
    let successor = fixture::install_ordinary(&mut fixture, &catalog, root.as_ref());
    let initial = fixture::initial_snapshot(&mut fixture, Arc::clone(&selected));
    let cell = product_cell(initial.clone(), &catalog, &fixture);
    let observation = cell
        .observe(&catalog, &fixture.owner)
        .expect("real selected-head observation");
    let history_after_observe = catalog.counters();
    let retention_after_observe = fixture.owner.cost_snapshot();
    let second = observation.clone();
    let final_clone = second.clone();
    assert_eq!(
        catalog.counters().direct_protection_acquisitions(),
        history_after_observe.direct_protection_acquisitions()
    );
    assert_eq!(
        fixture.owner.cost_snapshot().relational_contacts(),
        retention_after_observe.relational_contacts()
    );
    assert_eq!(
        fixture.owner.cost_snapshot().signal_contacts(),
        retention_after_observe.signal_contacts()
    );
    assert_eq!(fixture.owner.active_component_obligation_count(), 10);

    let successor_snapshot = fixture::successor_snapshot(&initial, Arc::clone(&successor));
    cell.compare_and_publish(
        &observation,
        fixture::product_head_protection(&fixture, &catalog, successor_snapshot),
    )
    .expect("move to sibling successor");
    let history_after_move = catalog.counters();
    assert_eq!(
        history_after_move.direct_protection_releases(),
        history_after_observe.direct_protection_releases() + 1
    );

    let reclaim_request = || {
        CompositeHistoryReclamationRequest::new(
            fixture.owner_identity,
            vec![selected.identity().clone()],
            1,
        )
    };
    let blocked = catalog
        .reclaim_batch(reclaim_request())
        .expect("live explicit observation protects selected sibling");
    assert_eq!(blocked.skipped_protected(), 1);
    drop(observation);
    drop(second);
    let blocked_until_final = catalog
        .reclaim_batch(reclaim_request())
        .expect("the final observation clone still protects selected sibling");
    assert_eq!(blocked_until_final.skipped_protected(), 1);
    let releases_before_final = catalog.counters().direct_protection_releases();
    let dependency_releases_before_final = fixture.owner.cost_snapshot().dependency_releases();
    drop(final_clone);
    assert_eq!(
        catalog.counters().direct_protection_releases(),
        releases_before_final + 1
    );
    assert_eq!(
        fixture.owner.cost_snapshot().dependency_releases(),
        dependency_releases_before_final + 2
    );
    assert_eq!(fixture.owner.active_component_obligation_count(), 8);
    let reclaimed = catalog
        .reclaim_batch(reclaim_request())
        .expect("final exact releases permit selected sibling reclamation");
    assert_eq!(
        reclaimed.reclaimed_commits(),
        &[selected.identity().clone()]
    );
}

#[test]
fn concurrent_observations_are_each_old_or_new_and_basis_coherent() {
    let (mut fixture, catalog, root) = fixture::installed_root();
    let selected = fixture::install_ordinary(&mut fixture, &catalog, root.as_ref());
    let successor = fixture::install_ordinary(&mut fixture, &catalog, root.as_ref());
    let initial = fixture::initial_snapshot(&mut fixture, Arc::clone(&selected));
    let cell = product_cell(initial.clone(), &catalog, &fixture);
    let expected = cell
        .observe(&catalog, &fixture.owner)
        .expect("real expected observation");
    let successor_snapshot = fixture::successor_snapshot(&initial, Arc::clone(&successor));
    let successor_protection =
        fixture::product_head_protection(&fixture, &catalog, successor_snapshot.clone());
    let gate = Arc::new(Barrier::new(2));
    let reader_cell = cell.clone();
    let reader_catalog = catalog.clone();
    let reader_owner = fixture.owner.clone();
    let reader_gate = Arc::clone(&gate);
    let reader_selected = selected.clone();
    let reader_successor = successor.clone();
    let reader_branch = initial.branch().clone();
    let reader_lifecycle = initial.lifecycle();
    let reader_old_generation = initial.generation();
    let reader_new_generation = successor_snapshot.generation();
    let reader = thread::spawn(move || {
        reader_gate.wait();
        for _ in 0..24 {
            let observation = reader_cell
                .observe(&reader_catalog, &reader_owner)
                .expect("real old-or-new observation");
            if observation.selected_commit() == reader_selected.identity() {
                assert_eq!(observation.branch_identity(), &reader_branch);
                assert_eq!(observation.lifecycle_incarnation(), reader_lifecycle);
                assert_eq!(observation.reference_generation(), reader_old_generation);
                assert!(
                    crate::basis::compare_exact(observation.basis(), reader_selected.basis())
                        .is_ok()
                );
            } else {
                assert_eq!(observation.selected_commit(), reader_successor.identity());
                assert_eq!(observation.branch_identity(), &reader_branch);
                assert_eq!(observation.lifecycle_incarnation(), reader_lifecycle);
                assert_eq!(observation.reference_generation(), reader_new_generation);
                assert!(
                    crate::basis::compare_exact(observation.basis(), reader_successor.basis())
                        .is_ok()
                );
            }
        }
    });
    gate.wait();
    cell.compare_and_publish(&expected, successor_protection)
        .expect("concurrent reader movement");
    reader.join().expect("reader does not observe a torn image");
    assert_eq!(
        cell.atomic_snapshot().commit().identity(),
        successor.identity()
    );
}
