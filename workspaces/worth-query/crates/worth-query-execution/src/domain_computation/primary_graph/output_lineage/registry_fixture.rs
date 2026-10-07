//! A real restored lineage slot for the registry's exact-index owner tests.

use std::any::TypeId;
use std::sync::Arc;

use super::{
    ProductCoordinate, RecordedSettlementIdentity, RecordedSourceIdentity, SemanticSource,
    WorthQueryApplicationOutputCorrespondence, WorthQueryApplicationOutputLineage,
};

struct RegistryRestoredOutput;

pub(in crate::domain_computation::primary_graph) fn recorded_settlement() -> (
    WorthQueryApplicationOutputLineage,
    Arc<RecordedSettlementIdentity>,
) {
    let (lineage, mut identities) = recorded_settlements(&[[0x71; 32]]);
    (lineage, identities.pop().unwrap())
}

pub(in crate::domain_computation::primary_graph) fn recorded_settlement_pair() -> (
    WorthQueryApplicationOutputLineage,
    [Arc<RecordedSettlementIdentity>; 2],
) {
    let (lineage, identities) = recorded_settlements(&[[0x71; 32], [0x74; 32]]);
    (lineage, identities.try_into().ok().unwrap())
}

fn recorded_settlements(
    partitions: &[[u8; 32]],
) -> (
    WorthQueryApplicationOutputLineage,
    Vec<Arc<RecordedSettlementIdentity>>,
) {
    let world =
        crate::domain_computation::primary_graph::tests::fixture::installed_authorization_world(
            true,
        );
    let product = world
        .application
        .product_runtime()
        .admit_product_branch(world.application.product_runtime().default_branch())
        .expect("fixture product occurrence is live");
    let observation = product.observation();
    let runtime_authority = world.application.runtime.authority_identity().as_u64();
    let schema = world.application.installed_schema.binding_identity();
    let entity = worth_relational::facade::identity::EntityId::new(
        worth_relational::facade::identity::PartitionId::main(),
        1,
        1,
    );
    let scope = crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding::from_entity(entity);
    let mut lineage = WorthQueryApplicationOutputLineage::default();
    for partition in partitions {
        lineage.record_restoration(
        TypeId::of::<RegistryRestoredOutput>(),
        runtime_authority,
        schema.clone(),
        scope,
        observation,
        Arc::new(WorthQueryApplicationOutputCorrespondence::default()),
        RecordedSourceIdentity::Checkpoint(
            crate::domain_computation::primary_graph::application_query::WorthQueryCheckpointSourceIdentity::new([0x72; 32]),
        ),
        *partition,
        None,
        [0x73; 32],
            crate::domain_computation::primary_graph::output_lineage::ComputationSourceEvidence::for_test(false).retain_facts(Arc::from([crate::domain_computation::primary_graph::WorthQueryApplicationObservedFact::SourceEntity {
            entity_id: entity,
        }])),
        None,
            None,
        );
    }
    let source = SemanticSource {
        runtime_authority,
        schema,
        scope,
        output_binding: TypeId::of::<RegistryRestoredOutput>(),
    };
    let coordinate = ProductCoordinate {
        occurrence: observation.lifecycle_incarnation(),
        generation: observation.reference_generation().get(),
    };
    let identities = partitions
        .iter()
        .map(|partition| {
            let (recorded, _) = lineage
                .latest_output_in_partition_budgeted(&source, coordinate, *partition, 1)
                .expect("recorded output is selectable");
            Arc::clone(
                &recorded
                    .expect("restored output has one actual lineage slot")
                    .settlement_identity,
            )
        })
        .collect();
    (lineage, identities)
}
