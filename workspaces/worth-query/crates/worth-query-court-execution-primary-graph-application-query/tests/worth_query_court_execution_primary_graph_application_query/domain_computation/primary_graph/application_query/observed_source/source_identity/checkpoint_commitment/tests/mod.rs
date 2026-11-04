use worth_relational::facade::identity::EntityId;
use worth_relational::facade::identity::PartitionId;

use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_query::observed_source::source_identity::WorthQueryObservedSourceEpoch;
    use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_query::observed_source::source_identity::WorthQueryObservedSourceMeaningRegistry;
    use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_query::observed_source::WorthQueryObservedSourceFootprint;
    use worth_query_execution::facade::domain_computation::WorthQueryApplicationBasisSelectionIdentity;

#[test]
fn portable_identity_ignores_runtime_occurrence_but_changes_with_source_meaning() {
    let root = EntityId::new(PartitionId::main(), 1, 1);
    let first_selection = product_selection();
    let second_selection = product_selection();
    let registry = WorthQueryObservedSourceMeaningRegistry::new(
            worth_query_execution::facade::domain_computation::WorthQueryRuntimeAuthorityIdentity::mint_for_test(),
        );
    let first = epoch(&registry, root, footprint(root, true), &first_selection);
    let second = epoch(&registry, root, footprint(root, true), &second_selection);

    assert_ne!(
        first.checkpoint_occurrence_for_test(),
        second.checkpoint_occurrence_for_test()
    );
    assert_eq!(first.checkpoint_identity(), second.checkpoint_identity());

    let changed = epoch(&registry, root, footprint(root, false), &first_selection);
    assert_ne!(first.checkpoint_identity(), changed.checkpoint_identity());

    let mut changed_selection = footprint(root, true);
    changed_selection.root_selection = Some(std::sync::Arc::new(
            worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_query::observed_source::root_selection::WorthQueryObservedRootSelection::new(
                worth_query_execution::court_doors_for_test::domain_computation::primary_graph::source_facts::SourceFacts { entities: vec![root], aspects: Vec::new(), adjacencies: Vec::new(), walks: None, derived: None, allocation: None }, None,
            ),
        ));
    let changed = epoch(&registry, root, changed_selection, &first_selection);
    assert_ne!(first.checkpoint_identity(), changed.checkpoint_identity());
}

pub fn epoch(
    registry: &WorthQueryObservedSourceMeaningRegistry,
    root: EntityId,
    footprint: WorthQueryObservedSourceFootprint,
    selection: &WorthQueryApplicationBasisSelectionIdentity,
) -> WorthQueryObservedSourceEpoch {
    let meaning = registry
        .intern_for_test(&[7; 32], &[8; 32], footprint, selection)
        .unwrap();
    WorthQueryObservedSourceEpoch::from_observation_for_test(&[7; 32], &[8; 32], root, selection, meaning)
        .unwrap()
}

pub fn footprint(root: EntityId, complete: bool) -> WorthQueryObservedSourceFootprint {
    WorthQueryObservedSourceFootprint {
        root,
        complete,
        entities: vec![root],
        aspects: Vec::new(),
        adjacencies: Vec::new(),
        root_selection: None,
    }
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
            worth_query_execution::facade::domain_computation::WorthQueryProductBranchReadIdentity::from_observation_for_test(
                product.observation_for_test(),
            ),
        )
}
