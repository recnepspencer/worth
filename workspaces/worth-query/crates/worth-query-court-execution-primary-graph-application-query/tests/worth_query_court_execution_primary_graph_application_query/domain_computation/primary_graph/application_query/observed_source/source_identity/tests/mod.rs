use worth_relational::facade::identity::EntityId;
use worth_relational::facade::identity::PartitionId;

use std::sync::Arc;
use std::collections::BTreeMap;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationBasisSelectionIdentity;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_query::observed_source::source_identity::WorthQueryObservedSourceEpoch;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_query::observed_source::WorthQueryObservedSourceFootprint;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_query::observed_source::source_identity::WorthQueryObservedSourceMeaningRegistry;

#[test]
fn refused_comparison_preserves_ordinal_and_live_coordinate_for_retry() {
    let registry = WorthQueryObservedSourceMeaningRegistry::new(
        worth_query_execution::facade::domain_computation::WorthQueryRuntimeAuthorityIdentity::mint_for_test(),
    );
    let root = EntityId::new(PartitionId::main(), 1, 1);
    // Descriptive registry mechanics need no runtime, Native read or epoch grant.
    let selection = WorthQueryApplicationBasisSelectionIdentity::Relational;
    let first = registry
        .intern_for_test(&[7; 32], &[8; 32], footprint(root, true), &selection)
        .unwrap();
    let ordinal = (*(*registry.state_for_test()).lock().unwrap().next_ordinal_for_test());
    let refusal = worth_relational::facade::transactions::RelationalExecutionDenialCause::PolicyMemoryExhausted {
        requested: 97,
        admitted: 53,
        ancestor: 11,
    };
    let denied = registry.intern_compared_for_test(
        &[7; 32],
        &[8; 32],
        footprint(root, false),
        &selection,
        &mut |_, _| Err(refusal),
    );
    assert_eq!(denied.err(), Some(refusal));
    {
        let state = (*registry.state_for_test()).lock().unwrap();
        assert_eq!((*state.next_ordinal_for_test()), ordinal);
        assert_eq!(state.meanings.len(), 1);
    }
    let retry = registry
        .intern_for_test(&[7; 32], &[8; 32], footprint(root, true), &selection)
        .unwrap();
    assert!(Arc::ptr_eq(&first, &retry));
    drop(first);
    drop(retry);
    assert!((*registry.state_for_test()).lock().unwrap().meanings.is_empty());
}

#[test]
fn exact_meaning_is_reused_but_a_changed_footprint_gets_a_new_identity() {
    let registry = WorthQueryObservedSourceMeaningRegistry::new(
        worth_query_execution::facade::domain_computation::WorthQueryRuntimeAuthorityIdentity::mint_for_test(),
    );
    let root = EntityId::new(PartitionId::main(), 1, 1);
    let selection = product_selection();
    let first = registry
        .intern_for_test(&[7; 32], &[8; 32], footprint(root, true), &selection)
        .unwrap();
    let equivalent = registry
        .intern_for_test(&[7; 32], &[8; 32], footprint(root, true), &selection)
        .unwrap();
    let changed = registry
        .intern_for_test(&[7; 32], &[8; 32], footprint(root, false), &selection)
        .unwrap();
    let returned = registry
        .intern_for_test(&[7; 32], &[8; 32], footprint(root, true), &selection)
        .unwrap();
    assert_eq!(first.identity_for_test(), equivalent.identity_for_test());
    assert_ne!(first.identity_for_test(), changed.identity_for_test());
    assert_ne!(
        first.runtime_idempotency_identity_for_test(),
        changed.runtime_idempotency_identity_for_test()
    );
    assert_eq!(first.identity_for_test(), returned.identity_for_test());
}

#[test]
fn result_set_membership_is_not_a_row_source_with_the_same_footprint() {
    let registry = WorthQueryObservedSourceMeaningRegistry::new(
        worth_query_execution::facade::domain_computation::WorthQueryRuntimeAuthorityIdentity::mint_for_test(),
    );
    let root = EntityId::new(PartitionId::main(), 1, 1);
    let selection = product_selection();
    let row = registry
        .intern_for_test(&[7; 32], &[8; 32], footprint(root, true), &selection)
        .unwrap();
    let set = registry
        .intern_result_set_for_test(&[7; 32], &[8; 32], footprint(root, true), &selection)
        .unwrap();
    assert_ne!(row.identity_for_test(), set.identity_for_test());
    assert_ne!(row.checkpoint_identity_for_test(), set.checkpoint_identity_for_test());
}

#[test]
fn epoch_rejects_a_meaning_from_another_source_coordinate() {
    let registry = WorthQueryObservedSourceMeaningRegistry::new(
        worth_query_execution::facade::domain_computation::WorthQueryRuntimeAuthorityIdentity::mint_for_test(),
    );
    let root = EntityId::new(PartitionId::main(), 1, 1);
    let selection = product_selection();
    let meaning = registry
        .intern_for_test(&[7; 32], &[8; 32], footprint(root, true), &selection)
        .unwrap();
    assert!(WorthQueryObservedSourceEpoch::from_observation_for_test(
        &[9; 32], &[8; 32], root, &selection, meaning,
    )
    .is_none());
}

#[test]
fn released_meaning_removes_only_its_fixed_coordinate() {
    let registry = WorthQueryObservedSourceMeaningRegistry::new(
        worth_query_execution::facade::domain_computation::WorthQueryRuntimeAuthorityIdentity::mint_for_test(),
    );
    let first_root = EntityId::new(PartitionId::main(), 1, 1);
    let second_root = EntityId::new(PartitionId::main(), 2, 1);
    let selection = product_selection();
    let first = registry
        .intern_for_test(&[7; 32], &[8; 32], footprint(first_root, true), &selection)
        .unwrap();
    let second = registry
        .intern_for_test(&[7; 32], &[8; 32], footprint(second_root, true), &selection)
        .unwrap();
    assert_eq!((*registry.state_for_test()).lock().unwrap().meanings.len(), 2);
    drop(first);
    assert_eq!((*registry.state_for_test()).lock().unwrap().meanings.len(), 1);
    assert_eq!((*second.footprint().root_for_test()), second_root);
}

#[test]
fn reinterned_source_retains_checkpoint_identity_without_reusing_runtime_idempotency() {
    let registry = WorthQueryObservedSourceMeaningRegistry::new(
        worth_query_execution::facade::domain_computation::WorthQueryRuntimeAuthorityIdentity::mint_for_test(),
    );
    let root = EntityId::new(PartitionId::main(), 1, 1);
    let selection = product_selection();
    let first = registry
        .intern_for_test(&[7; 32], &[8; 32], footprint(root, true), &selection)
        .unwrap();
    let first_runtime_identity = *first.identity_for_test();
    let first_commitment = first.runtime_idempotency_identity_for_test();
    let checkpoint_identity = first.checkpoint_identity_for_test();
    drop(first);

    let reinterned = registry
        .intern_for_test(&[7; 32], &[8; 32], footprint(root, true), &selection)
        .unwrap();
    assert_ne!(*reinterned.identity_for_test(), first_runtime_identity);
    assert_eq!(reinterned.runtime_idempotency_identity_for_test(), first_commitment);
    assert_eq!(reinterned.checkpoint_identity_for_test(), checkpoint_identity);
}

#[test]
fn sibling_product_occurrences_share_checkpoint_identity_not_runtime_idempotency() {
    let world =
        worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::world_installation::installed_authorization_world(
            true,
        );
    let source = world.application.current_world();
    world.application.with_host_advancement(|phase| {
let host_custody = phase.test_host_request_for_test();

let first_branch = world
            .application
            .branches()
            .fork(source)
            .components(|components| components.fork_relational().reuse_exact_signal_basis())
            .create(&host_custody.phase_for_application_for_test(&world.application))
            .expect("the first sibling product publishes");
        let second_branch = world
            .application
            .branches()
            .fork(source)
            .components(|components| components.fork_relational().reuse_exact_signal_basis())
            .create(&host_custody.phase_for_application_for_test(&world.application))
            .expect("the second sibling product publishes");
        let selection = |branch| {
            let selected = world
                .application
                .on_branch(branch)
                .select(&host_custody.phase_for_application_for_test(&world.application))
                .expect("the sibling product remains selectable");
            WorthQueryApplicationBasisSelectionIdentity::Product(
                worth_query_execution::facade::domain_computation::WorthQueryProductBranchReadIdentity::from_observation_for_test(
                    selected.product().observation_for_test(),
                ),
            )
        };
        let registry = WorthQueryObservedSourceMeaningRegistry::new(
        worth_query_execution::facade::domain_computation::WorthQueryRuntimeAuthorityIdentity::mint_for_test(),
    );
        let root = EntityId::new(PartitionId::main(), 1, 1);
        let first = registry
            .intern_for_test(
                &[7; 32],
                &[8; 32],
                footprint(root, true),
                &selection(first_branch),
            )
            .unwrap();
        let second = registry
            .intern_for_test(
                &[7; 32],
                &[8; 32],
                footprint(root, true),
                &selection(second_branch),
            )
            .unwrap();

        assert_ne!(
            first.runtime_idempotency_identity_for_test(),
            second.runtime_idempotency_identity_for_test()
        );
        assert_eq!(first.checkpoint_identity_for_test(), second.checkpoint_identity_for_test());

}).expect("the fixture block owns its installed host call")
}

#[test]
fn parameter_partitions_are_distinct_source_occurrences() {
    let root = EntityId::new(PartitionId::main(), 1, 1);
    let occurrence = product_selection();
    let registry = WorthQueryObservedSourceMeaningRegistry::new(
        worth_query_execution::facade::domain_computation::WorthQueryRuntimeAuthorityIdentity::mint_for_test(),
    );
    let lower_meaning = registry
        .intern_for_test(&[7; 32], &[1; 32], footprint(root, true), &occurrence)
        .unwrap();
    let upper_meaning = registry
        .intern_for_test(&[7; 32], &[2; 32], footprint(root, true), &occurrence)
        .unwrap();
    let lower = WorthQueryObservedSourceEpoch::from_observation_for_test(
        &[7; 32],
        &[1; 32],
        root,
        &occurrence,
        lower_meaning,
    )
    .unwrap();
    let upper = WorthQueryObservedSourceEpoch::from_observation_for_test(
        &[7; 32],
        &[2; 32],
        root,
        &occurrence,
        upper_meaning,
    )
    .unwrap();
    assert!(!lower.same_occurrence(&upper));
}

pub fn footprint(root: EntityId, complete: bool) -> WorthQueryObservedSourceFootprint {
    WorthQueryObservedSourceFootprint::new_for_test(root, complete, vec![root], Vec::new(), None, Vec::new())
}

pub fn product_selection() -> WorthQueryApplicationBasisSelectionIdentity {
    let world =
        worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::world_installation::installed_authorization_world(
            true,
        );
    let product = world
        .application
        .product_runtime()
        .admit_product_branch_for_test(world.application.product_runtime().default_branch_for_test())
        .expect("the fixture product occurrence is live");
    WorthQueryApplicationBasisSelectionIdentity::Product(
        worth_query_execution::facade::domain_computation::WorthQueryProductBranchReadIdentity::from_observation_for_test(product.observation_for_test()),
    )
}

#[test]
fn ordered_epoch_identity_survives_shared_meaning_registry_mutation() {
    use std::hash::DefaultHasher;
    use std::hash::Hash;
    use std::hash::Hasher;
    let hash = |epoch: &WorthQueryObservedSourceEpoch| {
        let mut state = DefaultHasher::new();
        epoch.hash(&mut state);
        state.finish()
    };
    let root = EntityId::new(PartitionId::main(), 1, 1);
    let occurrence = product_selection();
    let registry = WorthQueryObservedSourceMeaningRegistry::new(
        worth_query_execution::facade::domain_computation::WorthQueryRuntimeAuthorityIdentity::mint_for_test(),
    );
    let original = registry
        .intern_for_test(&[7; 32], &[8; 32], footprint(root, true), &occurrence)
        .unwrap();
    let epoch = WorthQueryObservedSourceEpoch::from_observation_for_test(
        &[7; 32],
        &[8; 32],
        root,
        &occurrence,
        original,
    )
    .unwrap();
    let records = BTreeMap::from([(epoch.clone(), "original")]);
    let before = hash(&epoch);
    // A second meaning mutates the shared registry without editing the first
    // epoch's coordinate or retained identity. Dropping it mutates it again.
    let changed = registry
        .intern_for_test(&[7; 32], &[8; 32], footprint(root, false), &occurrence)
        .unwrap();
    let changed = WorthQueryObservedSourceEpoch::from_observation_for_test(
        &[7; 32],
        &[8; 32],
        root,
        &occurrence,
        changed,
    )
    .unwrap();
    assert_ne!(epoch, changed);
    assert!(!records.contains_key(&changed));
    assert_eq!(hash(&epoch), before);
    assert_eq!(records.get(&epoch), Some(&"original"));
    drop(changed);
    assert_eq!(hash(&epoch), before);
    assert_eq!(records.get(&epoch), Some(&"original"));
}
