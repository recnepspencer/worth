use std::collections::BTreeMap;

use worth_foundational::facade::AspectFieldLocator;
use worth_query_installation::facade::ApplicationScalarValueBinding;
use worth_relational::facade::transactions::{
    AspectFieldPatch, EntityMutationIntent, MutationIntent, UpdateEntityFieldsIntent,
    WorkerIntentBatch,
};

use super::world_installation::AuthorizationWorld;
use super::{CapabilityIdentity, CapabilityStatus, CapabilityStatusField};
use crate::domain_computation::primary_graph::tests::fixture::live_scope;
use crate::domain_computation::primary_graph::WorthQueryPrincipalResolutionMode;

pub(in crate::domain_computation::primary_graph) fn revoke_current_capability(
    world: &AuthorizationWorld,
) {
    let grant = world
        .application
        .select_product_branch(world.application.product_runtime().default_branch())
        .expect("the selected product branch remains admitted")
        .resolve_entity(
            CapabilityIdentity::reference(),
            "capability-1".to_owned(),
            &live_scope(),
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap();
    let field = CapabilityStatusField::reference();
    let locator = installed_field(world, field.entity(), field.aspect(), field.field());
    let fields = AspectFieldPatch::from(BTreeMap::from([(
        locator,
        super::CapabilityStatusBinding::encode(&CapabilityStatus::Revoked).unwrap(),
    )]));
    super::publish_relational_mutation(
        world,
        WorkerIntentBatch::new("revoke-live-capability").push(MutationIntent::Entity(
            EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                entity_id: grant.entity_id(),
                fields,
            }),
        )),
    );
}

fn installed_field(
    world: &AuthorizationWorld,
    entity: &str,
    aspect: &str,
    field: &str,
) -> AspectFieldLocator {
    world
        .application
        .runtime
        .primary_graph()
        .unwrap()
        .layout()
        .field_locator(entity, aspect, field)
        .unwrap()
        .clone()
}
