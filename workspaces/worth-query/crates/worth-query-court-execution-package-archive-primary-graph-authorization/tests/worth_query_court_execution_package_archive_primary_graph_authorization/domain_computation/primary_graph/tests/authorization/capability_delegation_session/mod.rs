use std::num::NonZeroUsize;
use worth_query_execution::facade::domain_computation::WorthQueryOperationAuthorizationRefusalKind as AuthorizationRefusal;

use worth_query_declaration::facade::application_query::ApplicationQueryParameterSet;

use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::authenticated_principal;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::capability::CapabilityNotAfterField;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::capability_access_fixture::admit_touch_account_capability;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::capability_world_installation::installed_delegated_capability_world;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::live_scope;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::AccountIdentity;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::GovernedAccountOmissionQuery;
use crate::domain_computation::primary_graph::tests::authorization::capability_delegation_mutation::field;
use crate::domain_computation::primary_graph::tests::authorization::capability_delegation_mutation::update_grant_field;
use crate::domain_computation::primary_graph::tests::authorization::capability_progression::time;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationQueryAccessContext;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationQueryAdmissionDenialKind;
use worth_query_execution::facade::domain_computation::WorthQueryOperationAuthorizationDenialKind;
use worth_query_execution::facade::domain_computation::WorthQueryPrincipalResolutionMode;

#[test]
fn delegated_chain_expiry_between_access_and_governed_query_sessions_denies() {
    let world = installed_delegated_capability_world();
    world.authorization_time.script([time(100), time(102)]);
    world
        .application
        .with_host_advancement(|phase| {
            update_grant_field(
                &phase,
                &world,
                "capability-parent",
                field(&world, CapabilityNotAfterField::reference()),
                worth_foundational::facade::AspectValue::UInt64(101),
            )
        })
        .expect("the fixture read owns its installed host call");
    world
        .application
        .with_host_advancement(|phase| {
            update_grant_field(
                &phase,
                &world,
                "capability-child",
                field(&world, CapabilityNotAfterField::reference()),
                worth_foundational::facade::AspectValue::UInt64(101),
            )
        })
        .expect("the fixture read owns its installed host call");
    let request = live_scope();
    let principal = world
        .application
        .with_host_advancement(|phase| {
            authenticated_principal(&(phase).test_host_request(), &world, &request)
        })
        .expect("the fixture read owns its installed host call");
    let capability = world
        .application
        .with_host_advancement(|phase| {
            admit_touch_account_capability(
                &(phase).test_host_request(),
                &world,
                &principal,
                &request,
            )
            .expect("the narrowed chain is current at the first trusted sample")
        })
        .expect("the fixture read owns its installed host call");
    let account = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            world
                .application
                .select_product_branch(
                    &host_custody.borrowed_request_for_test(),
                    world.application.product_runtime().default_branch(),
                )
                .expect("the selected product branch remains admitted")
                .resolve_entity(
                    &phase,
                    AccountIdentity::reference(),
                    "account-1".to_owned(),
                    &request,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap()
        })
        .expect("the fixture read owns its installed host call");
    let query = world
        .application
        .installed_schema()
        .certification_query(GovernedAccountOmissionQuery::reference())
        .unwrap();
    let access = WorthQueryApplicationQueryAccessContext::new(&principal, &account);

    let Err(denial) = world
        .application
        .with_host_advancement(|phase| {
            world
                .selected_product(&(phase).test_host_request())
                .admit_governed_application_query(
                    &phase,
                    &query,
                    &access,
                    capability,
                    ApplicationQueryParameterSet::<GovernedAccountOmissionQuery>::new(),
                    worth_query_execution::facade::domain_computation::WorthQueryProductQueryControls::new(
                        NonZeroUsize::new(1).unwrap(),
                        &request,
                    ),
                )
        })
        .expect("the fixture read owns its installed host call")
    else {
        panic!("a fresh session must not reuse the earlier trusted time sample");
    };
    assert_eq!(
        denial.kind(),
        WorthQueryApplicationQueryAdmissionDenialKind::Authorization(
            WorthQueryOperationAuthorizationDenialKind::Refused(
                AuthorizationRefusal::CapabilityGrantMissing
            ),
        )
    );
    let authorization = denial
        .authorization_denial()
        .expect("governed readmission must retain its exact authorization denial");
    assert!(authorization.identity().is_some());
    assert_eq!(
        authorization.causes(),
        [WorthQueryOperationAuthorizationDenialKind::Refused(
            AuthorizationRefusal::CapabilityGrantMissing
        )]
    );
}
