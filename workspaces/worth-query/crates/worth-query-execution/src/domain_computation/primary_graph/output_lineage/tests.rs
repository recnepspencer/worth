use std::collections::BTreeMap;
use std::sync::Arc;

use super::{
    latest_output_in_partition, latest_output_matching, RecordedOutput, RecordedSourceIdentity,
    SemanticSource, WorthQueryApplicationOutputCorrespondence, WorthQueryApplicationOutputLineage,
    WorthQueryOutputSourcePosture,
};

#[test]
fn prior_output_selection_stays_with_its_parameter_partition() {
    let first_partition = [0x11; 32];
    let sibling_partition = [0x22; 32];
    let first = Arc::new(WorthQueryApplicationOutputCorrespondence::default());
    let sibling = Arc::new(WorthQueryApplicationOutputCorrespondence::default());
    let revised_first = Arc::new(WorthQueryApplicationOutputCorrespondence::default());
    let mut history = BTreeMap::new();
    history.insert(1, vec![record(Arc::clone(&first), first_partition)]);
    history.insert(2, vec![record(Arc::clone(&sibling), sibling_partition)]);
    history.insert(3, vec![record(Arc::clone(&revised_first), first_partition)]);

    let (_, selected_first_before_revision) =
        latest_output_in_partition(&history, 2, first_partition).unwrap();
    let (_, selected_sibling) = latest_output_in_partition(&history, 3, sibling_partition).unwrap();
    let (_, selected_first_after_revision) =
        latest_output_in_partition(&history, 3, first_partition).unwrap();

    assert!(Arc::ptr_eq(
        &selected_first_before_revision.correspondence,
        &first
    ));
    assert!(Arc::ptr_eq(&selected_sibling.correspondence, &sibling));
    assert!(Arc::ptr_eq(
        &selected_first_after_revision.correspondence,
        &revised_first
    ));
}

fn record(
    correspondence: Arc<WorthQueryApplicationOutputCorrespondence>,
    source_partition_identity: [u8; 32],
) -> RecordedOutput {
    RecordedOutput {
        correspondence,
        source_identity: Some(RecordedSourceIdentity::Runtime(
            crate::domain_computation::primary_graph::application_query::WorthQueryRuntimeSourceIdentity::new([0x33; 32]),
        )),
        source_partition_identity: Some(source_partition_identity),
        producer_dependency_identity: None,
        idempotency_key_identity: [0x44; 32],
        observed_source_facts: Arc::from([]),
    }
}

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
        Arc::from([]),
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
        Arc::from([]),
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
        Arc::from([]),
    );

    let source = SemanticSource {
        runtime_authority,
        schema,
        scope,
        output_binding: std::any::TypeId::of::<RestoredOutputBinding>(),
    };
    let history = &lineage.by_source[&source][&observation.lifecycle_incarnation()];
    let maximum_generation = observation.reference_generation().get();
    let (_, selected_first) =
        latest_output_in_partition(history, maximum_generation, first_partition).unwrap();
    let (_, selected_sibling) =
        latest_output_in_partition(history, maximum_generation, sibling_partition).unwrap();
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
    assert_eq!(
        lineage.source_posture_for_any_output_binding(
            runtime_authority,
            &source.schema,
            scope,
            observation.lifecycle_incarnation(),
            maximum_generation,
            &[std::any::TypeId::of::<RestoredOutputBinding>()],
            current_runtime,
            current_checkpoint,
        ),
        WorthQueryOutputSourcePosture::Exact(std::any::TypeId::of::<RestoredOutputBinding>())
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
#[should_panic(expected = "one restored output partition keeps one exact identity")]
fn restoration_rejects_conflicting_identity_for_the_same_partition() {
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
    let correspondence = Arc::new(WorthQueryApplicationOutputCorrespondence::default());
    let mut lineage = WorthQueryApplicationOutputLineage::default();

    for source_identity in [[0x31; 32], [0x32; 32]] {
        lineage.record_restoration(
            std::any::TypeId::of::<RestoredOutputBinding>(),
            runtime_authority,
            schema.clone(),
            scope,
            observation,
            Arc::clone(&correspondence),
            checkpoint_identity(source_identity),
            [0x11; 32],
            None,
            [0x41; 32],
            Arc::from([]),
        );
    }
}

fn checkpoint_identity(identity: [u8; 32]) -> RecordedSourceIdentity {
    RecordedSourceIdentity::Checkpoint(
        crate::domain_computation::primary_graph::application_query::WorthQueryCheckpointSourceIdentity::new(identity),
    )
}
