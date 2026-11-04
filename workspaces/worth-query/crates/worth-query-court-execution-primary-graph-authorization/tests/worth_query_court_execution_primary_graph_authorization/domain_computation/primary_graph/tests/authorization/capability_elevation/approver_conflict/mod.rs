use worth_query_execution::facade::domain_computation::WorthQueryOperationAuthorizationRefusalKind as AuthorizationRefusal;
use worth_relational::facade::identity::PartitionId;
use worth_relational::facade::symbols::ClientKey;
use worth_relational::facade::transactions::AspectFieldPatch;
use worth_relational::facade::transactions::CreateIntent;
use worth_relational::facade::transactions::EntityReference;
use worth_relational::facade::transactions::MutationIntent;
use worth_relational::facade::transactions::RelationSpec;
use worth_relational::facade::transactions::WorkerIntentBatch;

use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::authenticated_principal;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::capability::CapabilityConflictingBeneficiary;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::live_scope;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::AccountIdentity;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::ElevatedCapabilityTouchOperation;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::PrincipalIdentityField;
use crate::domain_computation::primary_graph::tests::authorization::capability_progression::time;
use worth_query_execution::facade::domain_computation::WorthQueryOperationAuthorizationDenialKind;
use worth_query_execution::facade::domain_computation::WorthQueryPrincipalResolutionMode;

#[test]
fn exact_conflicted_approver_cannot_open_elevation_authority() {
    let (world, request, approved) = crate::domain_computation::primary_graph::tests::authorization::capability_elevation::approval_transition::exact_approved_world();
    world.authorization_time.script([time(100)]);
    let principal = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request_for_test();
            authenticated_principal(&host_custody, &world, &request)
        })
        .expect("the fixture read owns its installed host call");
    world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request_for_test();
            add_approver_conflict(
                &host_custody,
                &host_custody.borrowed_request_for_test(),
                &world,
            )
        })
        .expect("the fixture read owns its installed host call");

    let Err(denial) = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request_for_test();
            crate::domain_computation::primary_graph::tests::authorization::capability_elevation::admit(
                &host_custody,
                &world,
                &approved,
                &principal,
                &request,
                Some("elevation-2"),
            )
        })
        .expect("the fixture read owns its installed host call")
    else {
        panic!("a conflict held by the exact approver must deny elevation admission");
    };

    assert_eq!(
        denial.kind(),
        WorthQueryOperationAuthorizationDenialKind::Refused(
            AuthorizationRefusal::ElevationApproverConflict
        )
    );
}

#[test]
fn approver_conflict_drift_stales_admitted_elevation_authority() {
    let (world, request, approved) = crate::domain_computation::primary_graph::tests::authorization::capability_elevation::approval_transition::exact_approved_world();
    world.authorization_time.script([time(100), time(100)]);
    let principal = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request_for_test();
            authenticated_principal(&host_custody, &world, &request)
        })
        .expect("the fixture read owns its installed host call");
    let access = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request_for_test();
            crate::domain_computation::primary_graph::tests::authorization::capability_elevation::admit(
                &host_custody,
                &world,
                &approved,
                &principal,
                &request,
                Some("elevation-2"),
            )
            .unwrap()
        })
        .expect("the fixture read owns its installed host call");
    world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request_for_test();
            add_approver_conflict(
                &host_custody,
                &host_custody.borrowed_request_for_test(),
                &world,
            )
        })
        .expect("the fixture read owns its installed host call");
    let operation = world
        .application
        .installed_schema()
        .installed_operation(ElevatedCapabilityTouchOperation::reference())
        .unwrap();

    let Err(denial) = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request_for_test();
            world.application.authorize_capability_operation(
                &host_custody.phase_for_application_for_test(&world.application),
                access,
                &operation,
                Default::default(),
            )
        })
        .expect("the fixture read owns its installed host call")
    else {
        panic!("new conflict truth for the exact approver must stale retained authority");
    };

    assert_eq!(
        denial.kind(),
        WorthQueryOperationAuthorizationDenialKind::Refused(
            AuthorizationRefusal::StaleAuthorization
        )
    );
}

pub fn add_approver_conflict(
    phase_custody: &worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::request_execution::advancement::test_host_policy::TestHostRequest<'_>,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::authorization_world_installation::AuthorizationWorld,
) {
    let scope = live_scope();
    let approver = world
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
            &phase_custody.phase_for_application_for_test(&world.application),
            PrincipalIdentityField::reference(),
            2_u64,
            &scope,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap();
    let account = world
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
            &phase_custody.phase_for_application_for_test(&world.application),
            AccountIdentity::reference(),
            "account-1".to_owned(),
            &scope,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap();
    let graph = (*world.application.runtime_for_test())
        .primary_graph_for_test()
        .unwrap();
    let relation_kind = (*graph
        .layout_for_test()
        .relation(CapabilityConflictingBeneficiary::reference().name())
        .unwrap()
        .kind_for_test());
    worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::product_publication::publish_relational_mutation(
        execution,
        world,
        WorkerIntentBatch::new("add-approver-conflict").push(MutationIntent::Create(
            CreateIntent::Relation(RelationSpec {
                partition_id: PartitionId::main(),
                kind_id: relation_kind,
                client_key: ClientKey::raw("elevation-approver-conflict-drift"),
                source: EntityReference::Existing(approver.entity_id_for_test()),
                target: EntityReference::Existing(account.entity_id_for_test()),
                fields: AspectFieldPatch::default(),
            }),
        )),
    );
}
