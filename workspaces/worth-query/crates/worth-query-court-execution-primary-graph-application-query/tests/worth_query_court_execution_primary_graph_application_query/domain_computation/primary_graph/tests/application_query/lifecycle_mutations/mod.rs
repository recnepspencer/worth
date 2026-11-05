use std::collections::BTreeMap;

use worth_query_declaration::facade::authentication::WorthQueryPrincipalMappingStatus;
use worth_query_declaration::facade::authentication::WorthQueryPrincipalMappingStatusBinding;
use worth_query_installation::facade::ApplicationScalarValueBinding;
use worth_relational::facade::identity::EntityId;
use worth_relational::facade::transactions::AspectFieldPatch;
use worth_relational::facade::transactions::DeleteRelationIntent;
use worth_relational::facade::transactions::EntityMutationIntent;
use worth_relational::facade::transactions::MutationIntent;
use worth_relational::facade::transactions::RelationMutationIntent;
use worth_relational::facade::transactions::UpdateEntityFieldsIntent;
use worth_relational::facade::transactions::WorkerIntentBatch;

use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::AccountOwner;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::authorization_world_installation::AuthorizationWorld;

pub fn disable_mapping(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &AuthorizationWorld,
    mapping_id: EntityId,
) {
    let graph = (*world.application.runtime_for_test())
        .primary_graph_for_test()
        .expect("test world publishes a primary graph");
    let layout = (*graph.layout_for_test())
        .principal_binding_for_test(world.binding.binding())
        .expect("test binding is installed")
        .clone();
    let fields = AspectFieldPatch::from(BTreeMap::from([(
        layout.status_locator_into_for_test(),
        WorthQueryPrincipalMappingStatusBinding::encode(
            &WorthQueryPrincipalMappingStatus::Disabled,
        )
        .unwrap(),
    )]));
    worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::product_publication::publish_relational_mutation(
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

pub fn revoke_account_ownership(
    phase_custody: &worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase<'_>,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &AuthorizationWorld,
    account: EntityId,
) {
    let graph = (*world.application.runtime_for_test())
        .primary_graph_for_test()
        .expect("test world publishes a primary graph");
    let relation_kind = *(*graph.layout_for_test())
        .relation(AccountOwner::reference().name())
        .expect("account ownership is installed")
        .kind_for_test();
    let selected = world.selected_product(&(phase_custody).test_host_request_for_test());
    let relation =
        graph
            .integration_handle_for_test()
            .with_runtime_for_test(execution, |runtime| {
                let snapshot = selected.application_basis_for_test().snapshot_handle();

                runtime
                    .read_truth()
                    .visible_relations_of_kind(execution, relation_kind, snapshot.version_id())
                    .expect("fixture read request remains live")
                    .into_iter()
                    .find(|record| record.target == account)
                    .expect("the admitted account has one ownership edge")
                    .relation_id
            });
    drop(selected);
    worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::product_publication::publish_relational_mutation(
        execution,
        world,
        WorkerIntentBatch::new("revoke-query-account-owner").push(MutationIntent::Relation(
            RelationMutationIntent::Delete(DeleteRelationIntent {
                relation_id: relation,
            }),
        )),
    );
}
