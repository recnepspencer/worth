use std::sync::Arc;

use super::{checkpoint_identity, RestoredOutputBinding};
use crate::domain_computation::primary_graph::output_lineage::{
    WorthQueryApplicationOutputCorrespondence, WorthQueryApplicationOutputLineage,
};

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
