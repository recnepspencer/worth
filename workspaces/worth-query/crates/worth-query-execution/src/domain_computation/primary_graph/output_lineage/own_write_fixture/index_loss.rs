//! An unrelated World publication evicts the derived index under a full ledger.
use crate::domain_computation::execution_runtime::WorthQueryInvalidationResources;
use crate::domain_computation::primary_graph::tests::{
    application_attempt::resolved_account,
    fixture::{live_scope, publish_relational_mutation, AccountLabel, AuthorizationWorld},
};
use worth_foundational::facade::{AspectValue, InternedString};
use worth_relational::facade::transactions::{
    AspectFieldPatch, EntityMutationIntent, MutationIntent, UpdateEntityFieldsIntent,
    WorkerIntentBatch,
};
pub(in crate::domain_computation::primary_graph) fn evict_index_with_unrelated_write(
    world: &AuthorizationWorld,
    resources: &WorthQueryInvalidationResources,
) {
    let unrelated = resolved_account(world, "unrelated", &live_scope());
    let graph = world.application.runtime.primary_graph().unwrap();
    let label = AccountLabel::reference();
    let label = graph
        .layout()
        .field_locator(label.entity(), label.aspect(), label.field())
        .unwrap()
        .clone();
    let held = resources
        .reserve_retained_capacity(
            resources.installation().maximum_retained_bytes - resources.retained_capacity_bytes(),
        )
        .unwrap();
    publish_relational_mutation(
        world,
        WorkerIntentBatch::new("unrelated-index-loss").push(MutationIntent::Entity(
            EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                entity_id: unrelated.entity_id(),
                fields: AspectFieldPatch::from(std::collections::BTreeMap::from([(
                    label,
                    AspectValue::String(InternedString::Raw("unrelated-write".to_owned())),
                )])),
            }),
        )),
    );
    drop(held);
}
