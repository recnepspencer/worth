use std::collections::BTreeMap;

use worth_query_declaration::facade::authentication::WorthQueryPrincipalMappingStatus;
use worth_query_installation::facade::TypedApplicationValue;
use worth_relational::facade::identity::EntityId;
use worth_relational::facade::transactions::{
    AspectFieldPatch, DeleteRelationIntent, EntityMutationIntent, MutationIntent,
    RelationMutationIntent, UpdateEntityFieldsIntent, WorkerIntentBatch,
};

use super::super::fixture::{AccountOwner, AuthorizationWorld};

pub(super) fn disable_mapping(world: &AuthorizationWorld, mapping_id: EntityId) {
    let graph = world
        .application
        .runtime
        .primary_graph()
        .expect("test world publishes a primary graph");
    let layout = graph
        .layout
        .principal_binding(world.binding.binding())
        .expect("test binding is installed")
        .clone();
    let fields = AspectFieldPatch::from(BTreeMap::from([(
        layout.status_locator,
        WorthQueryPrincipalMappingStatus::Disabled.into_foundational_value(),
    )]));
    super::super::fixture::publish_relational_mutation(
        world,
        WorkerIntentBatch::new("revoke-after-query-admission").push(MutationIntent::Entity(
            EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                entity_id: mapping_id,
                fields,
            }),
        )),
    );
}

pub(super) fn revoke_account_ownership(world: &AuthorizationWorld, account: EntityId) {
    let graph = world
        .application
        .runtime
        .primary_graph()
        .expect("test world publishes a primary graph");
    let relation_kind = graph
        .layout
        .relation(AccountOwner::reference().name())
        .expect("account ownership is installed")
        .kind;
    let selected = world.selected_product();
    let relation = graph.integration_handle().with_runtime_mut(|runtime| {
        let snapshot = selected.application_basis().snapshot_handle();
        let relation = runtime
            .read_truth()
            .visible_relations_of_kind(relation_kind, snapshot.version_id())
            .into_iter()
            .find(|record| record.target == account)
            .expect("the admitted account has one ownership edge")
            .relation_id;
        relation
    });
    drop(selected);
    super::super::fixture::publish_relational_mutation(
        world,
        WorkerIntentBatch::new("revoke-query-account-owner").push(MutationIntent::Relation(
            RelationMutationIntent::Delete(DeleteRelationIntent {
                relation_id: relation,
            }),
        )),
    );
}
