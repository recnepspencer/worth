use std::sync::Arc;

use super::{
    latest_output_matching, ProductCoordinate, RecordedSourceIdentity, SemanticSource,
    WorthQueryApplicationOutputCorrespondence, WorthQueryApplicationOutputLineage,
};

mod partition_selection;
mod restoration_identity;

struct RestoredOutputBinding;

#[test]
fn restoration_keeps_sibling_parameter_partitions_in_one_generation_slot() {
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
    let runtime_authority = world.application.runtime.authority_identity().as_u64();
    let schema = world.application.installed_schema.binding_identity();
    let scope = crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding::from_entity(
        worth_relational::facade::identity::EntityId::new(
            worth_relational::facade::identity::PartitionId::main(),
            1,
            1,
        ),
    );
    let first_partition = [0x11; 32];
    let sibling_partition = [0x22; 32];
    let first = Arc::new(WorthQueryApplicationOutputCorrespondence::default());
    let sibling = Arc::new(WorthQueryApplicationOutputCorrespondence::default());
    let mut lineage = WorthQueryApplicationOutputLineage::default();

    lineage.record_restoration(
        std::any::TypeId::of::<RestoredOutputBinding>(),
        runtime_authority,
        schema.clone(),
        scope,
        observation,
        Arc::clone(&first),
        checkpoint_identity([0x31; 32]),
        first_partition,
        None,
        [0x41; 32],
        source_facts(),
        None,
    );
    lineage.record_restoration(
        std::any::TypeId::of::<RestoredOutputBinding>(),
        runtime_authority,
        schema.clone(),
        scope,
        observation,
        Arc::clone(&sibling),
        checkpoint_identity([0x32; 32]),
        sibling_partition,
        Some([0x52; 32]),
        [0x42; 32],
        source_facts(),
        None,
    );
    lineage.record_restoration(
        std::any::TypeId::of::<RestoredOutputBinding>(),
        runtime_authority,
        schema.clone(),
        scope,
        observation,
        Arc::clone(&first),
        checkpoint_identity([0x31; 32]),
        first_partition,
        None,
        [0x41; 32],
        source_facts(),
        None,
    );

    let source = SemanticSource {
        runtime_authority,
        schema,
        scope,
        output_binding: std::any::TypeId::of::<RestoredOutputBinding>(),
    };
    let history = &lineage.by_source[&source][&observation.lifecycle_incarnation()];
    let maximum_generation = observation.reference_generation().get();
    let coordinate = ProductCoordinate {
        occurrence: observation.lifecycle_incarnation(),
        generation: maximum_generation,
    };
    let (selected_first, _) = lineage
        .latest_output_in_partition_budgeted(&source, coordinate, first_partition, 1)
        .unwrap();
    let (selected_sibling, _) = lineage
        .latest_output_in_partition_budgeted(&source, coordinate, sibling_partition, 1)
        .unwrap();
    let selected_first = selected_first.unwrap();
    let selected_sibling = selected_sibling.unwrap();
    let matching_sibling = latest_output_matching(history, maximum_generation, |recorded| {
        recorded.source_partition_identity == Some(sibling_partition)
    })
    .unwrap();

    assert!(Arc::ptr_eq(&selected_first.correspondence, &first));
    assert!(Arc::ptr_eq(&selected_sibling.correspondence, &sibling));
    assert!(Arc::ptr_eq(&matching_sibling.correspondence, &sibling));
    assert_eq!(history[&maximum_generation].len(), 2);

    let current_checkpoint =
        crate::domain_computation::primary_graph::application_query::WorthQueryCheckpointSourceIdentity::new([0x31; 32]);
    let current_runtime =
        crate::domain_computation::primary_graph::application_query::WorthQueryRuntimeSourceIdentity::new([0x99; 32]);
    let (candidates, _) = lineage
        .retained_output_candidates(
            runtime_authority,
            &source.schema,
            scope,
            observation.lifecycle_incarnation(),
            maximum_generation,
            &[std::any::TypeId::of::<RestoredOutputBinding>()],
            first_partition,
            8,
        )
        .unwrap();
    assert_eq!(candidates.len(), 1);
    assert_eq!(
        candidates[0].source_identity,
        Some(checkpoint_identity([0x31; 32]))
    );
    assert_eq!(
        lineage
            .qualified_output::<RestoredOutputBinding>(
                runtime_authority,
                &source.schema,
                scope,
                observation.lifecycle_incarnation(),
                maximum_generation,
                current_runtime,
                current_checkpoint,
            )
            .expect("the restored record matches its portable source identity")
            .source_identity,
        checkpoint_identity([0x31; 32]),
    );
}

#[test]
fn recovered_prior_correspondence_is_not_currentness_evidence_until_exact_readmission() {
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
    let runtime_authority = world.application.runtime.authority_identity().as_u64();
    let schema = world.application.installed_schema.binding_identity();
    let scope = crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding::from_entity(
        worth_relational::facade::identity::EntityId::new(
            worth_relational::facade::identity::PartitionId::main(),
            1,
            1,
        ),
    );
    let partition = [0x11; 32];
    let source_identity = checkpoint_identity([0x31; 32]);
    let runtime_identity = crate::domain_computation::primary_graph::application_query::WorthQueryRuntimeSourceIdentity::new([0x99; 32]);
    let checkpoint_identity = crate::domain_computation::primary_graph::application_query::WorthQueryCheckpointSourceIdentity::new([0x31; 32]);
    let correspondence = Arc::new(
        WorthQueryApplicationOutputCorrespondence::from_checkpoint_roles(
            std::any::TypeId::of::<RestoredOutputBinding>(),
            Vec::new(),
            |_| None,
        )
        .unwrap(),
    );
    let mut lineage = WorthQueryApplicationOutputLineage::default();

    lineage.record_recovered_prior_output(
        std::any::TypeId::of::<RestoredOutputBinding>(),
        runtime_authority,
        schema.clone(),
        scope,
        observation.lifecycle_incarnation(),
        observation.reference_generation().get(),
        Arc::clone(&correspondence),
        source_identity,
        partition,
        None,
        [0x41; 32],
        None,
    );
    lineage.record_recovered_prior_output(
        std::any::TypeId::of::<RestoredOutputBinding>(),
        runtime_authority,
        schema.clone(),
        scope,
        observation.lifecycle_incarnation(),
        observation.reference_generation().get() + 1,
        Arc::clone(&correspondence),
        source_identity,
        partition,
        None,
        [0x41; 32],
        None,
    );
    let semantic_source = SemanticSource {
        runtime_authority,
        schema: schema.clone(),
        scope,
        output_binding: std::any::TypeId::of::<RestoredOutputBinding>(),
    };
    assert_eq!(
        lineage.by_source[&semantic_source][&observation.lifecycle_incarnation()].len(),
        1,
        "repeated recovery must not become a newer output"
    );

    assert!(lineage
        .qualified_output::<RestoredOutputBinding>(
            runtime_authority,
            &schema,
            scope,
            observation.lifecycle_incarnation(),
            observation.reference_generation().get(),
            runtime_identity,
            checkpoint_identity,
        )
        .is_none());

    lineage.record_restoration(
        std::any::TypeId::of::<RestoredOutputBinding>(),
        runtime_authority,
        schema.clone(),
        scope,
        observation,
        Arc::clone(&correspondence),
        source_identity,
        partition,
        None,
        [0x41; 32],
        Arc::from([]),
        None,
    );

    assert!(
        lineage
            .qualified_output::<RestoredOutputBinding>(
                runtime_authority,
                &schema,
                scope,
                observation.lifecycle_incarnation(),
                observation.reference_generation().get(),
                runtime_identity,
                checkpoint_identity,
            )
            .is_none(),
        "an exact restored identity without consumed facts is not currentness evidence"
    );

    lineage.record_restoration(
        std::any::TypeId::of::<RestoredOutputBinding>(),
        runtime_authority,
        schema.clone(),
        scope,
        observation,
        correspondence,
        source_identity,
        partition,
        None,
        [0x41; 32],
        source_facts(),
        None,
    );

    assert!(lineage
        .qualified_output::<RestoredOutputBinding>(
            runtime_authority,
            &schema,
            scope,
            observation.lifecycle_incarnation(),
            observation.reference_generation().get(),
            runtime_identity,
            checkpoint_identity,
        )
        .is_some());
}

fn checkpoint_identity(identity: [u8; 32]) -> RecordedSourceIdentity {
    RecordedSourceIdentity::Checkpoint(
        crate::domain_computation::primary_graph::application_query::WorthQueryCheckpointSourceIdentity::new(identity),
    )
}

fn source_facts(
) -> Arc<[crate::domain_computation::primary_graph::WorthQueryApplicationObservedFact]> {
    Arc::from([
        crate::domain_computation::primary_graph::WorthQueryApplicationObservedFact::SourceEntity {
            entity_id: worth_relational::facade::identity::EntityId::new(
                worth_relational::facade::identity::PartitionId::main(),
                1,
                1,
            ),
        },
    ])
}
