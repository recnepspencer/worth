use std::{any::TypeId, sync::Arc};

use super::{checkpoint_identity, source_facts, RestoredOutputBinding};
use crate::domain_computation::primary_graph::output_lineage::{
    ProductCoordinate, RecordedOutput, SemanticSource, WorthQueryApplicationOutputCorrespondence,
    WorthQueryApplicationOutputLineage,
};

fn partition(number: u64) -> [u8; 32] {
    let mut partition = [0; 32];
    partition[..8].copy_from_slice(&number.to_le_bytes());
    partition
}

fn unrelated_partition_selection(population: u64) {
    let world =
        crate::domain_computation::primary_graph::tests::fixture::installed_authorization_world(
            true,
        );
    let product = world
        .application
        .product_runtime()
        .admit_product_branch(world.application.product_runtime().default_branch())
        .expect("the fixture's default product occurrence is live");
    let observation = product.observation();
    let occurrence = observation.lifecycle_incarnation();
    let runtime_authority = world.application.runtime.authority_identity().as_u64();
    let schema = world.application.installed_schema.binding_identity();
    let scope = crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding::from_entity(
        worth_relational::facade::identity::EntityId::new(
            worth_relational::facade::identity::PartitionId::main(),
            1,
            1,
        ),
    );
    let source = SemanticSource {
        runtime_authority,
        schema: schema.clone(),
        scope,
        output_binding: TypeId::of::<RestoredOutputBinding>(),
    };
    let target = Arc::new(WorthQueryApplicationOutputCorrespondence::default());
    let sibling = Arc::new(WorthQueryApplicationOutputCorrespondence::default());
    let mut lineage = WorthQueryApplicationOutputLineage::default();
    lineage.record_recovered_prior_output(
        source.output_binding,
        runtime_authority,
        schema.clone(),
        scope,
        occurrence,
        1,
        Arc::clone(&target),
        checkpoint_identity([0x31; 32]),
        partition(0),
        None,
        [0x41; 32],
        None,
    );
    for number in 1..=population {
        lineage.record_recovered_prior_output(
            source.output_binding,
            runtime_authority,
            schema.clone(),
            scope,
            occurrence,
            number + 1,
            Arc::clone(&sibling),
            checkpoint_identity([0x31; 32]),
            partition(number),
            None,
            [0x41; 32],
            None,
        );
    }
    let coordinate = ProductCoordinate {
        occurrence,
        generation: population + 1,
    };
    assert!(lineage
        .latest_output_in_partition_budgeted(&source, coordinate, partition(0), 0)
        .is_err());
    let (selected, work) = lineage
        .latest_output_in_partition_budgeted(&source, coordinate, partition(0), 1)
        .expect("one indexed coordinate fits the fixed budget");
    assert_eq!(work, 1);
    assert!(Arc::ptr_eq(&selected.unwrap().correspondence, &target));
    let (candidates, work) = lineage
        .retained_output_candidates(
            runtime_authority,
            &schema,
            scope,
            occurrence,
            population + 1,
            &[source.output_binding],
            partition(0),
            1,
        )
        .expect("unrelated partitions cannot exhaust one selection lookup");
    assert_eq!(work, 1);
    assert_eq!(candidates.len(), 1);
    assert!(Arc::ptr_eq(&candidates[0].correspondence, &target));
    let (missing, work) = lineage
        .latest_output_in_partition_budgeted(&source, coordinate, [0xff; 32], 1)
        .expect("a missing partition also costs one lookup");
    assert!(missing.is_none());
    assert_eq!(work, 1);

    let revised_target = Arc::new(WorthQueryApplicationOutputCorrespondence::default());
    let revised_generation = population + 2;
    let records = lineage
        .by_source
        .get_mut(&source)
        .unwrap()
        .get_mut(&occurrence)
        .unwrap()
        .entry(revised_generation)
        .or_default();
    let slot = records.len();
    records.push(RecordedOutput {
        correspondence: Arc::clone(&revised_target),
        source_identity: Some(checkpoint_identity([0x31; 32])),
        source_partition_identity: Some(partition(0)),
        producer_dependency_identity: None,
        idempotency_key_identity: [0x42; 32],
        observed_source_facts: None,
        resources: None,
    });
    lineage.partition_index.insert(
        source.clone(),
        occurrence,
        revised_generation,
        partition(0),
        slot,
    );
    let revised_coordinate = ProductCoordinate {
        occurrence,
        generation: revised_generation,
    };
    assert!(lineage
        .matching_output_in_partition_budgeted(
            &source,
            revised_coordinate,
            partition(0),
            1,
            |recorded| Arc::ptr_eq(&recorded.correspondence, &target),
        )
        .is_err());
    let (prior, work) = lineage
        .matching_output_in_partition_budgeted(
            &source,
            revised_coordinate,
            partition(0),
            2,
            |recorded| Arc::ptr_eq(&recorded.correspondence, &target),
        )
        .expect("same-partition predecessors, not siblings, consume the budget");
    assert_eq!(work, 2);
    assert!(Arc::ptr_eq(&prior.unwrap().correspondence, &target));

    // An unavailable derived locator must never guess from a sibling record.
    lineage.partition_index = Default::default();
    let (unavailable, work) = lineage
        .latest_output_in_partition_budgeted(&source, revised_coordinate, partition(0), 1)
        .expect("missing index data is a bounded miss");
    assert!(unavailable.is_none());
    assert_eq!(work, 1);

    lineage
        .by_source
        .get_mut(&source)
        .unwrap()
        .get_mut(&occurrence)
        .unwrap()
        .get_mut(&1)
        .unwrap()[0]
        .source_partition_identity = None;
    assert!(lineage
        .matching_legacy_output_budgeted(
            &source,
            revised_coordinate,
            population as usize + 1,
            |recorded| Arc::ptr_eq(&recorded.correspondence, &target),
        )
        .is_err());
    let (legacy, work) = lineage
        .matching_legacy_output_budgeted(
            &source,
            revised_coordinate,
            population as usize + 2,
            |recorded| Arc::ptr_eq(&recorded.correspondence, &target),
        )
        .expect("legacy unpartitioned correspondence remains discoverable within its budget");
    assert_eq!(work, population as usize + 2);
    assert!(Arc::ptr_eq(&legacy.unwrap().correspondence, &target));
}

#[test]
fn retained_output_selection_has_fixed_work_with_1k_and_10k_unrelated_partitions() {
    for population in [1_000, 10_000] {
        unrelated_partition_selection(population);
    }
}

#[test]
#[ignore = "100k output partitions are a scheduled scale court; run with --ignored"]
fn retained_output_selection_has_fixed_work_with_100k_unrelated_partitions() {
    unrelated_partition_selection(100_000);
}

#[test]
fn fork_selection_uses_ancestor_partition_and_retirement_prunes_its_locator() {
    use worth_runtime_world::facade::{
        ProductBranchCreationIntent, ProductBranchCreationPlans, RelationalBranchCreationPlan,
        RuntimeWorldBranchCreationOutcome, RuntimeWorldCancellationSource,
        SignalBranchCreationPlan,
    };

    let world =
        crate::domain_computation::primary_graph::tests::fixture::installed_authorization_world(
            true,
        );
    let product = world
        .application
        .product_runtime()
        .admit_product_branch(world.application.product_runtime().default_branch())
        .expect("the fixture's default product occurrence is live");
    let parent = product.observation();
    let intent = ProductBranchCreationIntent::from_source(
        "partition-index-fork",
        ProductBranchCreationPlans::new(
            RelationalBranchCreationPlan::ReuseExact,
            SignalBranchCreationPlan::ReuseExact,
        ),
    )
    .expect("the fork intent is valid");
    let RuntimeWorldBranchCreationOutcome::Performed(child) = world
        .application
        .product_runtime()
        .create_product_branch(
            &product,
            None,
            intent,
            &RuntimeWorldCancellationSource::new().token(),
        )
        .expect("the product owner admits the fork")
    else {
        panic!("the product owner must create the fork");
    };
    let runtime_authority = world.application.runtime.authority_identity().as_u64();
    let schema = world.application.installed_schema.binding_identity();
    let scope = crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding::from_entity(
        worth_relational::facade::identity::EntityId::new(
            worth_relational::facade::identity::PartitionId::main(),
            1,
            1,
        ),
    );
    let source = SemanticSource {
        runtime_authority,
        schema: schema.clone(),
        scope,
        output_binding: TypeId::of::<RestoredOutputBinding>(),
    };
    let correspondence = Arc::new(WorthQueryApplicationOutputCorrespondence::default());
    let mut lineage = WorthQueryApplicationOutputLineage::default();
    lineage.record_restoration(
        source.output_binding,
        runtime_authority,
        schema.clone(),
        scope,
        parent,
        Arc::clone(&correspondence),
        checkpoint_identity([0x31; 32]),
        partition(0),
        None,
        [0x41; 32],
        source_facts(),
        None,
    );
    lineage.register_fork(parent, &child);
    let (candidates, work) = lineage
        .retained_output_candidates(
            runtime_authority,
            &schema,
            scope,
            child.lifecycle_incarnation(),
            child.reference_generation().get(),
            &[source.output_binding],
            partition(0),
            2,
        )
        .expect("a fork traverses its parent coordinate");
    assert_eq!(work, 2);
    assert!(Arc::ptr_eq(&candidates[0].correspondence, &correspondence));

    lineage.release_occurrence(parent.lifecycle_incarnation());
    assert!(
        lineage
            .partition_index
            .latest(
                &source,
                ProductCoordinate {
                    occurrence: parent.lifecycle_incarnation(),
                    generation: parent.reference_generation().get(),
                },
                partition(0),
            )
            .is_some(),
        "a live child retains its ancestor's output locator"
    );
    lineage.release_occurrence(child.lifecycle_incarnation());
    assert!(
        lineage
            .partition_index
            .latest(
                &source,
                ProductCoordinate {
                    occurrence: parent.lifecycle_incarnation(),
                    generation: parent.reference_generation().get(),
                },
                partition(0),
            )
            .is_none(),
        "retired lineage must prune its derived locator"
    );
    assert!(!lineage.by_source.contains_key(&source));
}
