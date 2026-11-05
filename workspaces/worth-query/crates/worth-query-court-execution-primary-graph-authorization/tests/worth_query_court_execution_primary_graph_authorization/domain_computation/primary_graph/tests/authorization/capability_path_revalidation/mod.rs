use worth_query_execution::facade::domain_computation::WorthQueryOperationAuthorizationRefusalKind as AuthorizationRefusal;
use worth_relational::facade::identity::EntityId;
use worth_relational::facade::identity::PartitionId;
use worth_relational::facade::symbols::ClientKey;
use worth_relational::facade::transactions::AspectFieldPatch;
use worth_relational::facade::transactions::CreateIntent;
use worth_relational::facade::transactions::DeleteRelationIntent;
use worth_relational::facade::transactions::EntityReference;
use worth_relational::facade::transactions::MutationIntent;
use worth_relational::facade::transactions::RelationMutationIntent;
use worth_relational::facade::transactions::RelationSpec;
use worth_relational::facade::transactions::WorkerIntentBatch;

use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::authenticated_principal;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::capability::CapabilityCustodian;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::capability::CapabilityGrantor;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::capability_world_installation::installed_capability_authorization_world;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::live_scope;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::CapabilityIdentity;
use crate::domain_computation::primary_graph::tests::authorization::capability_progression::admitted_capability_operation;
use crate::domain_computation::primary_graph::tests::authorization::capability_progression::time;
use worth_query_execution::facade::domain_computation::WorthQueryOperationAuthorizationDenialKind;
use worth_query_execution::facade::domain_computation::WorthQueryPrincipalResolutionMode;

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
            authenticated_principal(&(phase).test_host_request_for_test(), &world, &request)
        })
        .expect("the fixture read owns its installed host call");
    let mut admission = world
        .application
        .with_host_advancement(|phase| {
            admitted_capability_operation(
                &(phase).test_host_request_for_test(),
                &world,
                &principal,
                &request,
            )
        })
        .expect("the fixture read owns its installed host call");
    let commit_authorization = admission
        .take_authorization_dependencies_for_test(
            (*(*world.application.runtime_for_test()).authorization_for_test()).bridge_for_test(),
        )
        .unwrap();

    world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request_for_test();
            replace_grantor_with_custodian(
                &phase,
                &host_custody.borrowed_request_for_test(),
                &world,
                principal.principal_entity_id_for_test(),
            )
        })
        .expect("the fixture read owns its installed host call");

    let commit_lane = (*(*world.application.runtime_for_test()).primary_provider_for_test())
        .application_branch_commit_lane_for_test(
            admission
                .graph_work_for_test()
                .mutation_product()
                .unwrap()
                .observation_for_test(),
        );
    let coordination = commit_lane.enter_for_test();
    let Err(denial) = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request_for_test();
            commit_authorization.authorize_application_commit_for_test(
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

pub fn replace_grantor_with_custodian(
    phase_custody: &worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase<'_>,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::authorization_world_installation::AuthorizationWorld,
    principal: EntityId,
) {
    let request = live_scope();
    let grant = world
        .application
        .select_product_branch_for_test(
            execution,
            world
                .application
                .product_runtime()
                .default_branch_for_test(),
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
    let graph = (*world.application.runtime_for_test())
        .primary_graph_for_test()
        .unwrap();
    let grantor_kind = (*graph
        .layout_for_test()
        .relation(CapabilityGrantor::reference().name())
        .unwrap()
        .kind_for_test());
    let custodian_kind = (*graph
        .layout_for_test()
        .relation(CapabilityCustodian::reference().name())
        .unwrap()
        .kind_for_test());
    let handle = graph.integration_handle_for_test();
    let selected = world.selected_product(&(phase_custody).test_host_request_for_test());
    let grantor = handle.with_runtime_for_test(execution, |runtime| {
        let snapshot = selected.application_basis_for_test().snapshot_handle();
        runtime
            .read_truth()
            .visible_relations_of_kind(execution, grantor_kind, snapshot.version_id())
            .expect("fixture read request remains live")
            .into_iter()
            .find(|record| {
                record.source == principal && record.target == grant.entity_id_for_test()
            })
            .expect("the admitted capability has one current grantor path")
            .relation_id
    });
    drop(selected);
    worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::product_publication::publish_relational_mutation(
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
                    target: EntityReference::Existing(grant.entity_id_for_test()),
                    fields: AspectFieldPatch::default(),
                },
            ))),
    );
}
