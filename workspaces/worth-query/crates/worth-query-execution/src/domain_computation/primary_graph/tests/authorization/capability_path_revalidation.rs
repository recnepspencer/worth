use crate::domain_computation::authorization::WorthQueryOperationAuthorizationRefusalKind as AuthorizationRefusal;
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
    let principal = world
        .application
        .with_host_advancement(|phase| {
            authenticated_principal(&(phase).test_host_request(), &world, &request)
        })
        .expect("the fixture read owns its installed host call");
    let mut admission = world
        .application
        .with_host_advancement(|phase| {
            admitted_capability_operation(
                &(phase).test_host_request(),
                &world,
                &principal,
                &request,
            )
        })
        .expect("the fixture read owns its installed host call");
    let commit_authorization = admission
        .take_authorization_dependencies(world.application.authorization.bridge())
        .unwrap();

    world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            replace_grantor_with_custodian(
                &phase,
                &host_custody.borrowed_request_for_test(),
                &world,
                principal.principal_entity_id(),
            )
        })
        .expect("the fixture read owns its installed host call");

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
    let Err(denial) = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            commit_authorization.authorize_application_commit(
                &host_custody.borrowed_request_for_test(),
                &world.application,
                &admission,
                &coordination,
            )
        })
        .expect("the fixture read owns its installed host call")
    else {
        panic!("a replacement policy path must not inherit retained commit authority");
    };
    assert_eq!(
        denial.kind(),
        WorthQueryOperationAuthorizationDenialKind::Refused(
            AuthorizationRefusal::StaleAuthorization
        )
    );
}

fn replace_grantor_with_custodian(
    phase_custody: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &super::super::fixture::AuthorizationWorld,
    principal: EntityId,
) {
    let request = live_scope();
    let grant = world
        .application
        .select_product_branch(
            execution,
            world.application.product_runtime().default_branch(),
        )
        .expect("the selected product branch remains admitted")
        .resolve_entity(
            phase_custody,
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
    let selected = world.selected_product(&(phase_custody).test_host_request());
    let grantor = handle.with_runtime_mut(execution, |runtime| {
        let snapshot = selected.application_basis().snapshot_handle();
        runtime
            .read_truth()
            .visible_relations_of_kind(execution, grantor_kind, snapshot.version_id())
            .expect("fixture read request remains live")
            .into_iter()
            .find(|record| record.source == principal && record.target == grant.entity_id())
            .expect("the admitted capability has one current grantor path")
            .relation_id
    });
    drop(selected);
    super::super::fixture::publish_relational_mutation(
        execution,
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
