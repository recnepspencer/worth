use worth_relational::facade::identity::{EntityId, PartitionId};

use super::*;

#[test]
fn exact_meaning_is_reused_but_a_changed_footprint_gets_a_new_identity() {
    let registry = WorthQueryObservedSourceMeaningRegistry::new(17);
    let root = EntityId::new(PartitionId::main(), 1, 1);
    let selection = product_selection();
    let first = registry
        .intern(&[7; 32], &[8; 32], footprint(root, true), &selection)
        .unwrap();
    let equivalent = registry
        .intern(&[7; 32], &[8; 32], footprint(root, true), &selection)
        .unwrap();
    let changed = registry
        .intern(&[7; 32], &[8; 32], footprint(root, false), &selection)
        .unwrap();
    let returned = registry
        .intern(&[7; 32], &[8; 32], footprint(root, true), &selection)
        .unwrap();
    assert_eq!(first.identity(), equivalent.identity());
    assert_ne!(first.identity(), changed.identity());
    assert_eq!(first.identity(), returned.identity());
}

#[test]
fn released_meaning_removes_only_its_fixed_coordinate() {
    let registry = WorthQueryObservedSourceMeaningRegistry::new(18);
    let first_root = EntityId::new(PartitionId::main(), 1, 1);
    let second_root = EntityId::new(PartitionId::main(), 2, 1);
    let selection = product_selection();
    let first = registry
        .intern(&[7; 32], &[8; 32], footprint(first_root, true), &selection)
        .unwrap();
    let second = registry
        .intern(&[7; 32], &[8; 32], footprint(second_root, true), &selection)
        .unwrap();
    assert_eq!(registry.state.lock().unwrap().meanings.len(), 2);
    drop(first);
    assert_eq!(registry.state.lock().unwrap().meanings.len(), 1);
    assert_eq!(second.footprint().root, second_root);
}

#[test]
fn reinterned_source_retains_durable_idempotency_identity() {
    let registry = WorthQueryObservedSourceMeaningRegistry::new(20);
    let root = EntityId::new(PartitionId::main(), 1, 1);
    let selection = product_selection();
    let first = registry
        .intern(&[7; 32], &[8; 32], footprint(root, true), &selection)
        .unwrap();
    let first_runtime_identity = *first.identity();
    let durable_identity = first.durable_idempotency_identity();
    drop(first);

    let reinterned = registry
        .intern(&[7; 32], &[8; 32], footprint(root, true), &selection)
        .unwrap();
    assert_ne!(*reinterned.identity(), first_runtime_identity);
    assert_eq!(reinterned.durable_idempotency_identity(), durable_identity);
}

#[test]
fn parameter_partitions_are_distinct_source_occurrences() {
    let root = EntityId::new(PartitionId::main(), 1, 1);
    let occurrence = product_selection();
    let registry = WorthQueryObservedSourceMeaningRegistry::new(19);
    let lower_meaning = registry
        .intern(&[7; 32], &[1; 32], footprint(root, true), &occurrence)
        .unwrap();
    let upper_meaning = registry
        .intern(&[7; 32], &[2; 32], footprint(root, true), &occurrence)
        .unwrap();
    let lower = WorthQueryObservedSourceEpoch::from_observation(
        &[7; 32],
        &[1; 32],
        root,
        &occurrence,
        lower_meaning,
    )
    .unwrap();
    let upper = WorthQueryObservedSourceEpoch::from_observation(
        &[7; 32],
        &[2; 32],
        root,
        &occurrence,
        upper_meaning,
    )
    .unwrap();
    assert!(!lower.same_occurrence(&upper));
}

fn footprint(root: EntityId, complete: bool) -> WorthQueryObservedSourceFootprint {
    WorthQueryObservedSourceFootprint {
        root,
        complete,
        entities: vec![root],
        aspects: Vec::new(),
        adjacencies: Vec::new(),
    }
}

fn product_selection() -> WorthQueryApplicationBasisSelectionIdentity {
    let world =
        crate::domain_computation::primary_graph::tests::fixture::installed_authorization_world(
            true,
        );
    let product = world
        .application
        .product_runtime()
        .admit_product_branch(world.application.product_runtime().default_branch())
        .expect("the fixture product occurrence is live");
    WorthQueryApplicationBasisSelectionIdentity::Product(
        crate::basis::WorthQueryProductBranchReadIdentity::from_observation(product.observation()),
    )
}
