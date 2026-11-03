//! A real restored lineage slot for the registry's exact-index owner tests.
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_binding_identity::OutputBindingIdentity;

use std::sync::Arc;
use worth_query_execution::court_doors_for_test::equality_type_id::EqualityTypeId as TypeId;

use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::ProductCoordinate;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::settlement_identity::RecordedSettlementIdentity;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::recorded_source_identity::RecordedSourceIdentity;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::semantic_source::SemanticSource;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::effect_program::output_correspondence::WorthQueryApplicationOutputCorrespondence;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::WorthQueryApplicationOutputLineage;

pub struct RegistryRestoredOutput;

pub fn recorded_settlement() -> (
    WorthQueryApplicationOutputLineage,
    Arc<RecordedSettlementIdentity>,
) {
    let (lineage, mut identities) = recorded_settlements(&[[0x71; 32]]);
    (lineage, identities.pop().unwrap())
}

pub fn recorded_settlement_pair() -> (
    WorthQueryApplicationOutputLineage,
    [Arc<RecordedSettlementIdentity>; 2],
) {
    let (lineage, identities) = recorded_settlements(&[[0x71; 32], [0x74; 32]]);
    (lineage, identities.try_into().ok().unwrap())
}

pub fn recorded_settlements(
    partitions: &[[u8; 32]],
) -> (
    WorthQueryApplicationOutputLineage,
    Vec<Arc<RecordedSettlementIdentity>>,
) {
    let world =
        crate::domain_computation::primary_graph::tests::fixture::world_installation::installed_authorization_world(
            true,
        );
    let product = world
        .application
        .product_runtime()
        .admit_product_branch_for_test(world.application.product_runtime().default_branch_for_test())
        .expect("fixture product occurrence is live");
    let observation = product.observation_for_test();
    let runtime_authority = (*world.application.runtime_for_test()).authority_identity().as_u64();
    let schema = (*world.application.installed_schema_for_test()).binding_identity();
    let entity = worth_relational::facade::identity::EntityId::new(
        worth_relational::facade::identity::PartitionId::main(),
        1,
        1,
    );
    let scope = worth_query_execution::facade::domain_computation::WorthQueryOperationScopeEntityBinding::from_entity_for_test(entity);
    let mut lineage = WorthQueryApplicationOutputLineage::default();
    lineage.fixture_binding(
        TypeId::of::<RegistryRestoredOutput>(),
        "RegistryRestoredOutput",
    );

    for partition in partitions {
        lineage.record_restoration(
        TypeId::of::<RegistryRestoredOutput>(),
        runtime_authority,
        schema.clone(),
        scope,
        observation,
        Arc::new(WorthQueryApplicationOutputCorrespondence::default()),
        RecordedSourceIdentity::Checkpoint(
            worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_query::observed_source::source_identity::identity_kind::WorthQueryCheckpointSourceIdentity::new_for_test([0x72; 32]),
        ),
        *partition,
        None,
        [0x73; 32],
            worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::computation_source::ComputationSourceEvidence::for_test(false).retain_facts_for_test(Arc::from([worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::fact::WorthQueryApplicationObservedFact::SourceEntity {
            entity_id: entity,
        }])),
            None,
         None,
         None,
     );
    }
    let source = SemanticSource {
        runtime_authority,
        schema,
        scope,
        output_binding: OutputBindingIdentity::declared_for_test("RegistryRestoredOutput"),
    };
    let coordinate = ProductCoordinate {
        occurrence: observation.lifecycle_incarnation(),
        generation: observation.reference_generation().get(),
    };
    let identities = partitions
        .iter()
        .map(|partition| {
            let recorded = lineage.latest_output_in_partition_for_test(&source, coordinate, *partition);
            Arc::clone(
                &(*recorded
                    .expect("restored output has one actual lineage slot")
                    .settlement_identity_for_test()),
            )
        })
        .collect();
    (lineage, identities)
}
