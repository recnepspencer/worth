use worth_relational::facade::identity::{EntityId, PartitionId};
use worth_relational::facade::symbols::ClientKey;
use worth_relational::facade::transactions::{
    AspectFieldPatch, CreateIntent, DeleteRelationIntent, EntityReference, MutationIntent,
    RelationMutationIntent, RelationSpec, WorkerIntentBatch,
};

use super::super::application_attempt::authenticated_principal;
use super::super::fixture::capability::{CapabilityCustodian, CapabilityGrantor};
use super::super::fixture::{
    installed_capability_authorization_world, live_scope, CapabilityIdentity,
};
use super::capability_progression::{admitted_capability_operation, time};
use crate::domain_computation::primary_graph::{
    WorthQueryOperationAuthorizationDenialKind, WorthQueryPrincipalResolutionMode,
};

#[test]
fn final_commit_rejects_a_replacement_policy_path_for_the_same_grant() {
    let world = installed_capability_authorization_world();
    world
        .authorization_time
        .script([time(100), time(100), time(100)]);
    let request = live_scope();
    let principal = authenticated_principal(&world, &request);
    let mut admission = admitted_capability_operation(&world, &principal, &request);
    let commit_authorization = admission
        .take_authorization_dependencies(world.application.authorization.bridge())
        .unwrap();

    replace_grantor_with_custodian(&world, principal.principal_entity_id());

    let commit_lane = world
        .application
        .primary_provider
        .application_branch_commit_lane(
            admission
                .graph_work()
                .mutation_product()
                .unwrap()
                .observation(),
        );
    let coordination = commit_lane.enter();
    let Err(denial) = commit_authorization.authorize_application_commit(
        &world.application,
        &admission,
        &coordination,
    ) else {
        panic!("a replacement policy path must not inherit retained commit authority");
    };
    assert_eq!(
        denial.kind(),
        WorthQueryOperationAuthorizationDenialKind::StaleAuthorization
    );
}

fn replace_grantor_with_custodian(
    world: &super::super::fixture::AuthorizationWorld,
    principal: EntityId,
) {
    let request = live_scope();
    let grant = world
        .application
        .select_product_branch(world.application.product_runtime().default_branch())
        .expect("the selected product branch remains admitted")
        .resolve_entity(
            CapabilityIdentity::reference(),
            "capability-1".to_owned(),
            &request,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap();
    let graph = world.application.runtime.primary_graph().unwrap();
    let grantor_kind = graph
        .layout()
        .relation(CapabilityGrantor::reference().name())
        .unwrap()
        .kind;
    let custodian_kind = graph
        .layout()
        .relation(CapabilityCustodian::reference().name())
        .unwrap()
        .kind;
    let handle = graph.integration_handle();
    let selected = world.selected_product();
    let grantor = handle.with_runtime_mut(|runtime| {
        let snapshot = selected.application_basis().snapshot_handle();
        runtime
            .read_truth()
            .visible_relations_of_kind(grantor_kind, snapshot.version_id())
            .into_iter()
            .find(|record| record.source == principal && record.target == grant.entity_id())
            .expect("the admitted capability has one current grantor path")
            .relation_id
    });
    drop(selected);
    super::super::fixture::publish_relational_mutation(
        world,
        WorkerIntentBatch::new("replace-capability-policy-path")
            .push(MutationIntent::Relation(RelationMutationIntent::Delete(
                DeleteRelationIntent {
                    relation_id: grantor,
                },
            )))
            .push(MutationIntent::Create(CreateIntent::Relation(
                RelationSpec {
                    partition_id: PartitionId::main(),
                    kind_id: custodian_kind,
                    client_key: ClientKey::raw("capability-1-custodian"),
                    source: EntityReference::Existing(principal),
                    target: EntityReference::Existing(grant.entity_id()),
                    fields: AspectFieldPatch::default(),
                },
            ))),
    );
}
