use std::collections::BTreeMap;

use worth_query_declaration::facade::authentication::{
    WorthQueryPrincipalMappingStatus, WorthQueryPrincipalMappingStatusBinding,
};
use worth_query_installation::facade::ApplicationScalarValueBinding;
use worth_relational::facade::identity::EntityId;
use worth_relational::facade::transactions::{
    AspectFieldPatch, DeleteRelationIntent, EntityMutationIntent, MutationIntent,
    RelationMutationIntent, UpdateEntityFieldsIntent, WorkerIntentBatch,
};

use super::super::fixture::{AccountOwner, AuthorizationWorld};

pub(super) fn disable_mapping(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &AuthorizationWorld,
    mapping_id: EntityId,
) {
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
        WorthQueryPrincipalMappingStatusBinding::encode(
            &WorthQueryPrincipalMappingStatus::Disabled,
        )
        .unwrap(),
    )]));
    super::super::fixture::publish_relational_mutation(
        execution,
        world,
        WorkerIntentBatch::new("revoke-after-query-admission").push(MutationIntent::Entity(
            EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                entity_id: mapping_id,
                fields,
            }),
        )),
    );
}

pub(super) fn revoke_account_ownership(
    phase_custody: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &AuthorizationWorld,
    account: EntityId,
) {
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
    let selected = world.selected_product(&(phase_custody).test_host_request());
    let relation = graph
        .integration_handle()
        .with_runtime_mut(execution, |runtime| {
            let snapshot = selected.application_basis().snapshot_handle();
            let relation = runtime
                .read_truth()
                .visible_relations_of_kind(execution, relation_kind, snapshot.version_id())
                .into_iter()
                .find(|record| record.target == account)
                .expect("the admitted account has one ownership edge")
                .relation_id;
            relation
        });
    drop(selected);
    super::super::fixture::publish_relational_mutation(
        execution,
        world,
        WorkerIntentBatch::new("revoke-query-account-owner").push(MutationIntent::Relation(
            RelationMutationIntent::Delete(DeleteRelationIntent {
                relation_id: relation,
            }),
        )),
    );
}
