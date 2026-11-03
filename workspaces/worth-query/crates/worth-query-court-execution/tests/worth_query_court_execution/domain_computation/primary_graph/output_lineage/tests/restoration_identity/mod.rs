use std::sync::Arc;
use worth_query_execution::equality_type_id::EqualityTypeId as TypeId;

use crate::domain_computation::primary_graph::output_lineage::tests::checkpoint_identity;
use crate::domain_computation::primary_graph::output_lineage::tests::RestoredOutputBinding;
use worth_query_execution::domain_computation::primary_graph::application_attempt::effect_program::output_correspondence::WorthQueryApplicationOutputCorrespondence;
use worth_query_execution::domain_computation::primary_graph::output_lineage::WorthQueryApplicationOutputLineage;

#[test]
#[should_panic(expected = "one restored output partition keeps one exact identity")]
fn restoration_rejects_conflicting_identity_for_the_same_partition() {
    let world =
        worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::world_installation::installed_authorization_world(
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
    let scope = worth_query_execution::facade::domain_computation::WorthQueryOperationScopeEntityBinding::from_entity(
        worth_relational::facade::identity::EntityId::new(
            worth_relational::facade::identity::PartitionId::main(),
            1,
            1,
        ),
    );
    let correspondence = Arc::new(WorthQueryApplicationOutputCorrespondence::default());
    let mut lineage = WorthQueryApplicationOutputLineage::default();
    lineage.fixture_binding(
        TypeId::of::<RestoredOutputBinding>(),
        "RestoredOutputBinding",
    );

    for source_identity in [[0x31; 32], [0x32; 32]] {
        lineage.record_restoration(
            TypeId::of::<RestoredOutputBinding>(),
            runtime_authority,
            schema.clone(),
            scope,
            observation,
            Arc::clone(&correspondence),
            checkpoint_identity(source_identity),
            [0x11; 32],
            None,
            [0x41; 32],
            worth_query_execution::domain_computation::primary_graph::output_lineage::computation_source::ComputationSourceEvidence::for_test(false).retain_facts(Arc::<[worth_query_execution::domain_computation::primary_graph::application_attempt::fact::WorthQueryApplicationObservedFact]>::from([])),
            None,
            None,
            None,
        );
    }
}
