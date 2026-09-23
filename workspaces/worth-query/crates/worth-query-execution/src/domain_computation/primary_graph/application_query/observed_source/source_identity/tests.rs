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
    assert_ne!(
        first.runtime_idempotency_identity(),
        changed.runtime_idempotency_identity()
    );
    assert_eq!(first.identity(), returned.identity());
}

#[test]
fn result_set_membership_is_not_a_row_source_with_the_same_footprint() {
    let registry = WorthQueryObservedSourceMeaningRegistry::new(19);
    let root = EntityId::new(PartitionId::main(), 1, 1);
    let selection = product_selection();
    let row = registry
        .intern(&[7; 32], &[8; 32], footprint(root, true), &selection)
        .unwrap();
    let set = registry
        .intern_result_set(&[7; 32], &[8; 32], footprint(root, true), &selection)
        .unwrap();
    assert_ne!(row.identity(), set.identity());
    assert_ne!(row.checkpoint_identity(), set.checkpoint_identity());
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
fn reinterned_source_retains_checkpoint_identity_without_reusing_runtime_idempotency() {
    let registry = WorthQueryObservedSourceMeaningRegistry::new(20);
    let root = EntityId::new(PartitionId::main(), 1, 1);
    let selection = product_selection();
    let first = registry
        .intern(&[7; 32], &[8; 32], footprint(root, true), &selection)
        .unwrap();
    let first_runtime_identity = *first.identity();
    let first_commitment = first.runtime_idempotency_identity();
    let checkpoint_identity = first.checkpoint_identity();
    drop(first);

    let reinterned = registry
        .intern(&[7; 32], &[8; 32], footprint(root, true), &selection)
        .unwrap();
    assert_ne!(*reinterned.identity(), first_runtime_identity);
    assert_eq!(reinterned.runtime_idempotency_identity(), first_commitment);
    assert_eq!(reinterned.checkpoint_identity(), checkpoint_identity);
}

#[test]
fn sibling_product_occurrences_share_checkpoint_identity_not_runtime_idempotency() {
    let world =
        crate::domain_computation::primary_graph::tests::fixture::installed_authorization_world(
            true,
        );
    let source = world.application.current_world();
    let first_branch = world
        .application
        .branches()
        .fork(source)
        .components(|components| components.fork_relational().reuse_exact_signal_basis())
        .create()
        .expect("the first sibling product publishes");
    let second_branch = world
        .application
        .branches()
        .fork(source)
        .components(|components| components.fork_relational().reuse_exact_signal_basis())
        .create()
        .expect("the second sibling product publishes");
    let selection = |branch| {
        let selected = world
            .application
            .on_branch(branch)
            .select()
            .expect("the sibling product remains selectable");
        WorthQueryApplicationBasisSelectionIdentity::Product(
            crate::basis::WorthQueryProductBranchReadIdentity::from_observation(
                selected.product().observation(),
            ),
        )
    };
    let registry = WorthQueryObservedSourceMeaningRegistry::new(21);
    let root = EntityId::new(PartitionId::main(), 1, 1);
    let first = registry
        .intern(
            &[7; 32],
            &[8; 32],
            footprint(root, true),
            &selection(first_branch),
        )
        .unwrap();
    let second = registry
        .intern(
            &[7; 32],
            &[8; 32],
            footprint(root, true),
            &selection(second_branch),
        )
        .unwrap();

    assert_ne!(
        first.runtime_idempotency_identity(),
        second.runtime_idempotency_identity()
    );
    assert_eq!(first.checkpoint_identity(), second.checkpoint_identity());
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
        root_selection: None,
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
