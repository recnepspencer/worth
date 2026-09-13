use std::sync::{mpsc, Arc, Barrier};
use std::thread;
use std::time::Duration;

use super::protection::ProductBranchHeadProtectionDenial;
use super::{
    ProductBranchHeadProtection, ProductBranchReferenceCell, ProductBranchReferenceCellDenial,
    ProductBranchReferenceSnapshot,
};
use crate::branch::observation::ProductBranchObservation;
use crate::branch::reference_test_fixture as fixture;
use crate::history::CompositeHistoryCatalog;
use crate::identity::ProductBranchReferenceGeneration;

fn product_cell(
    snapshot: ProductBranchReferenceSnapshot,
    catalog: &CompositeHistoryCatalog,
    fixture: &fixture::RealReferenceFixture,
) -> ProductBranchReferenceCell {
    let protection = fixture::product_head_protection(fixture, catalog, snapshot);
    ProductBranchReferenceCell::new(protection).expect("protected cell admission")
}

fn observation(
    cell: &ProductBranchReferenceCell,
    fixture: &fixture::RealReferenceFixture,
    catalog: &CompositeHistoryCatalog,
) -> ProductBranchObservation {
    cell.observe(catalog, &fixture.owner)
        .expect("real managed observation")
}

#[test]
fn construction_requires_exact_product_head_proof_and_returns_it_on_denial() {
    let (mut fixture, catalog, root) = fixture::installed_root();
    let snapshot = fixture::initial_snapshot(&mut fixture, Arc::clone(&root));
    let other = fixture::install_ordinary(&mut fixture, &catalog, root.as_ref());
    let before = catalog.counters();
    let wrong_publication = fixture
        .owner
        .issue_publication(other.basis())
        .expect("other publication retention");
    let wrong_transfer = wrong_publication
        .into_product_head_transfer(other.basis())
        .expect("other product-head transfer");
    let wrong_history = catalog
        .protect_product_head(other.as_ref())
        .expect("other commit is installed");
    let wrong_protection =
        ProductBranchHeadProtection::owner_issued(snapshot.clone(), wrong_transfer, wrong_history)
            .expect_err("the mismatched protection is returned intact")
            .into_protection();
    let wrong = ProductBranchReferenceCell::new(wrong_protection)
        .expect_err("a proof for another commit cannot admit the cell");
    assert_eq!(
        wrong.denial(),
        ProductBranchHeadProtectionDenial::HistoryCommitMismatch
    );
    drop(wrong.into_protection());
    assert_eq!(
        catalog.counters().direct_protection_releases(),
        before.direct_protection_releases() + 1
    );

    let (mut foreign_fixture, foreign_catalog, foreign_root) = fixture::installed_root();
    let foreign_snapshot =
        fixture::initial_snapshot(&mut foreign_fixture, Arc::clone(&foreign_root));
    let foreign_publication = foreign_fixture
        .owner
        .issue_publication(foreign_snapshot.commit().basis())
        .expect("foreign publication retention");
    let foreign_transfer = foreign_publication
        .into_product_head_transfer(foreign_snapshot.commit().basis())
        .expect("foreign product-head transfer");
    let foreign_history = foreign_catalog
        .protect_product_head(foreign_snapshot.commit())
        .expect("foreign commit is installed in its catalog");
    let foreign_protection = ProductBranchHeadProtection::owner_issued(
        snapshot.clone(),
        foreign_transfer,
        foreign_history,
    )
    .expect_err("the foreign protection is returned intact")
    .into_protection();
    let foreign = ProductBranchReferenceCell::new(foreign_protection)
        .expect_err("a foreign proof cannot admit the cell");
    assert_eq!(
        foreign.denial(),
        ProductBranchHeadProtectionDenial::SnapshotOwnerMismatch
    );
    drop(foreign.into_protection());

    let cell = product_cell(snapshot, &catalog, &fixture);
    assert_eq!(cell.atomic_snapshot().commit().identity(), root.identity());
}

#[test]
fn stale_expected_head_precedes_successor_validation_and_returns_proof() {
    let (mut fixture, catalog, root) = fixture::installed_root();
    let initial = fixture::initial_snapshot(&mut fixture, Arc::clone(&root));
    let cell = product_cell(initial.clone(), &catalog, &fixture);
    let expected = observation(&cell, &fixture, &catalog);
    let first = fixture::install_ordinary(&mut fixture, &catalog, root.as_ref());
    let first_snapshot = fixture::successor_snapshot(&initial, Arc::clone(&first));
    cell.compare_and_publish(
        &expected,
        fixture::product_head_protection(&fixture, &catalog, first_snapshot.clone()),
    )
    .expect("first movement");

    let malformed_commit = fixture::install_ordinary(&mut fixture, &catalog, root.as_ref());
    let malformed = fixture::initial_snapshot(&mut fixture, Arc::clone(&malformed_commit));
    let before_stale_failure = catalog.counters();
    let stale = cell
        .compare_and_publish(
            &expected,
            fixture::product_head_protection(&fixture, &catalog, malformed),
        )
        .expect_err("stale expected head wins before malformed successor");
    assert!(matches!(
        stale.denial(),
        ProductBranchReferenceCellDenial::ExpectedHeadMismatch(mismatch)
            if mismatch
                .axes()
                .contains(&crate::branch::observation::ProductBranchObservationMismatchAxis::SelectedCompositeCommit)
    ));
    assert_eq!(stale.observed_head(), &first_snapshot);
    assert_eq!(
        stale.observed_head().reference_generation(),
        first_snapshot.generation()
    );
    assert_eq!(
        stale.observed_head().selected_commit(),
        first_snapshot.commit().identity()
    );
    assert!(crate::basis::compare_exact(
        stale.observed_head().basis(),
        first_snapshot.commit().basis()
    )
    .is_ok());

    let first_observation = observation(&cell, &fixture, &catalog);
    let second = fixture::install_ordinary(&mut fixture, &catalog, first.as_ref());
    let second_snapshot = fixture::successor_snapshot(&first_snapshot, Arc::clone(&second));
    cell.compare_and_publish(
        &first_observation,
        fixture::product_head_protection(&fixture, &catalog, second_snapshot.clone()),
    )
    .expect("a later movement succeeds while the failure retains its exact CAS winner");
    assert_eq!(
        stale.observed_head(),
        &first_snapshot,
        "the failure must not follow the cell to a later head"
    );
    let stale_protection = stale.into_successor_protection();
    let (_stale_snapshot, stale_product_head, stale_history, stale_receipt) =
        stale_protection.into_parts();
    assert!(stale_receipt.is_some());
    assert_eq!(
        stale_product_head.relational().dependency(),
        crate::retention::ComponentBasisDependencyClass::ProductBranchHead
    );
    let active_before_recovery_drop = fixture.owner.active_component_obligation_count();
    drop(stale_receipt);
    drop(stale_history);
    drop(stale_product_head);
    assert_eq!(
        fixture.owner.active_component_obligation_count(),
        active_before_recovery_drop - 2,
        "CAS loss custody releases the returned pair only after the caller drops it"
    );
    assert_eq!(
        catalog.counters().direct_protection_releases(),
        before_stale_failure.direct_protection_releases() + 2,
        "one release belongs to the later winning movement and one to the rejected successor"
    );
    assert_eq!(cell.atomic_snapshot(), second_snapshot);

    let current = observation(&cell, &fixture, &catalog);
    let malformed_commit = fixture::install_ordinary(&mut fixture, &catalog, root.as_ref());
    let malformed =
        fixture::initial_snapshot_named(&mut fixture, Arc::clone(&malformed_commit), "elsewhere");
    let before_successor_failure = catalog.counters();
    let invalid = cell
        .compare_and_publish(
            &current,
            fixture::product_head_protection(&fixture, &catalog, malformed),
        )
        .expect_err("successor branch mismatch is denied");
    assert!(matches!(
        invalid.denial(),
        ProductBranchReferenceCellDenial::SuccessorBranchMismatch
    ));
    drop(invalid.into_successor_protection());
    assert_eq!(
        catalog.counters().direct_protection_releases(),
        before_successor_failure.direct_protection_releases() + 1
    );
}

#[test]
fn while_current_holds_the_guard_across_its_section_and_refuses_a_displaced_head() {
    let (mut fixture, catalog, root) = fixture::installed_root();
    let initial = fixture::initial_snapshot(&mut fixture, Arc::clone(&root));
    let cell = product_cell(initial.clone(), &catalog, &fixture);
    let expected = observation(&cell, &fixture, &catalog);
    let successor = fixture::install_ordinary(&mut fixture, &catalog, root.as_ref());
    let successor_snapshot = fixture::successor_snapshot(&initial, Arc::clone(&successor));
    let successor_protection =
        fixture::product_head_protection(&fixture, &catalog, successor_snapshot.clone());

    assert!(!cell.writers_are_locked_out_for_test());
    let section = cell.while_current(&expected, 7, |argument| {
        assert!(
            cell.writers_are_locked_out_for_test(),
            "no publication can move the head while the section runs"
        );
        argument + 1
    });
    assert_eq!(section.ok(), Some(8));
    assert!(!cell.writers_are_locked_out_for_test());

    cell.compare_and_publish(&expected, successor_protection)
        .expect("the head moves once the section has returned");
    let (observed, argument) = cell
        .while_current(&expected, 7, |_| {
            panic!("a displaced head enters no section")
        })
        .expect_err("the section is refused against the moved head");
    assert_eq!(observed.commit().identity(), successor.identity());
    assert_eq!(argument, 7, "the refused argument comes back untouched");

    let shared = cell.clone();
    assert!(
        shared.into_protection().is_none(),
        "a shared cell keeps its protection"
    );
    assert!(
        cell.into_protection().is_some(),
        "the last holder gets the protection back"
    );
    drop(expected);
}

#[test]
fn bootstrap_product_head_protection_survives_without_an_observation() {
    let (mut fixture, catalog, root) = fixture::installed_root();
    let snapshot = fixture::initial_snapshot(&mut fixture, Arc::clone(&root));
    let protection = fixture::bootstrap_product_head_protection(&fixture, &catalog, snapshot);
    assert_eq!(fixture.owner.active_component_obligation_count(), 4);

    let cell = ProductBranchReferenceCell::new(protection).expect("direct bootstrap protection");
    assert_eq!(fixture.owner.active_component_obligation_count(), 4);
    let observation = cell
        .observe(&catalog, &fixture.owner)
        .expect("observation owns an independent pair");
    assert_eq!(fixture.owner.active_component_obligation_count(), 6);
    drop(observation);
    assert_eq!(fixture.owner.active_component_obligation_count(), 4);
    drop(cell);
    assert_eq!(fixture.owner.active_component_obligation_count(), 2);
    assert_eq!(fixture.owner.reclaim(2).reclaimed(), 0);
    drop(catalog);
    assert_eq!(fixture.owner.active_component_obligation_count(), 0);
    assert_eq!(fixture.owner.reclaim(2).reclaimed(), 2);
}

#[test]
fn real_same_head_contention_has_one_winner_and_no_torn_image() {
    let (mut fixture, catalog, root) = fixture::installed_root();
    let initial = fixture::initial_snapshot(&mut fixture, Arc::clone(&root));
    let cell = product_cell(initial.clone(), &catalog, &fixture);
    let expected = Arc::new(observation(&cell, &fixture, &catalog));
    let first = fixture::install_ordinary(&mut fixture, &catalog, root.as_ref());
    let second = fixture::install_ordinary(&mut fixture, &catalog, root.as_ref());
    let first_snapshot = fixture::successor_snapshot(&initial, Arc::clone(&first));
    let second_snapshot = fixture::successor_snapshot(&initial, Arc::clone(&second));
    let first_protection =
        fixture::product_head_protection(&fixture, &catalog, first_snapshot.clone());
    let second_protection =
        fixture::product_head_protection(&fixture, &catalog, second_snapshot.clone());
    let start = Arc::new(Barrier::new(3));
    let (results, receive) = mpsc::channel();

    let first_cell = cell.clone();
    let first_expected = Arc::clone(&expected);
    let first_start = Arc::clone(&start);
    let first_results = results.clone();
    let first_worker = thread::spawn(move || {
        first_start.wait();
        first_results
            .send(
                first_cell
                    .compare_and_publish(first_expected.as_ref(), first_protection)
                    .is_ok(),
            )
            .expect("first contention result");
    });
    let second_cell = cell.clone();
    let second_expected = Arc::clone(&expected);
    let second_start = Arc::clone(&start);
    let second_worker = thread::spawn(move || {
        second_start.wait();
        results
            .send(
                second_cell
                    .compare_and_publish(second_expected.as_ref(), second_protection)
                    .is_ok(),
            )
            .expect("second contention result");
    });
    start.wait();
    let first_won = receive
        .recv_timeout(Duration::from_secs(1))
        .expect("first contention worker completes");
    let second_won = receive
        .recv_timeout(Duration::from_secs(1))
        .expect("second contention worker completes");
    first_worker.join().expect("first contention worker");
    second_worker.join().expect("second contention worker");
    assert_ne!(first_won, second_won);

    let selected = cell.atomic_snapshot();
    assert_eq!(selected.generation().get(), 1);
    assert!(
        selected.commit().identity() == first.identity()
            || selected.commit().identity() == second.identity()
    );
    assert!(crate::basis::compare_exact(selected.commit().basis(), &fixture.basis).is_ok());
    assert_eq!(
        selected.generation(),
        ProductBranchReferenceGeneration::initial()
            .advance()
            .unwrap()
    );
}

#[test]
fn unrelated_product_reference_progress_does_not_wait() {
    let (mut fixture, catalog, root) = fixture::installed_root();
    let first_snapshot = fixture::initial_snapshot(&mut fixture, Arc::clone(&root));
    let second_snapshot = fixture::initial_snapshot(&mut fixture, Arc::clone(&root));
    let first = product_cell(first_snapshot, &catalog, &fixture);
    let second = product_cell(second_snapshot.clone(), &catalog, &fixture);
    let expected = observation(&second, &fixture, &catalog);
    let successor = fixture::install_ordinary(&mut fixture, &catalog, root.as_ref());
    let successor_snapshot = fixture::successor_snapshot(&second_snapshot, Arc::clone(&successor));
    let successor_protection =
        fixture::product_head_protection(&fixture, &catalog, successor_snapshot.clone());
    let held = first.hold_for_test();
    let (completed, receive) = mpsc::channel();
    let worker = thread::spawn(move || {
        completed
            .send(
                second
                    .compare_and_publish(&expected, successor_protection)
                    .is_ok(),
            )
            .expect("unrelated completion result");
    });
    assert!(receive
        .recv_timeout(Duration::from_secs(1))
        .expect("unrelated cell progresses while first is locked"));
    drop(held);
    worker.join().expect("unrelated worker");
}
