use std::num::NonZeroUsize;

use worth_query_declaration::facade::application_query::ApplicationQueryParameterSet;

use super::super::application_attempt::authenticated_principal;
use super::super::fixture::capability::CapabilityNotAfterField;
use super::super::fixture::{
    admit_touch_account_capability, installed_delegated_capability_world, live_scope,
    AccountIdentity, GovernedAccountOmissionQuery,
};
use super::capability_delegation_mutation::{field, update_grant_field};
use super::capability_progression::time;
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationQueryAccessContext, WorthQueryApplicationQueryAdmissionDenialKind,
    WorthQueryOperationAuthorizationDenialKind, WorthQueryPrincipalResolutionMode,
};

#[test]
fn delegated_chain_expiry_between_access_and_governed_query_sessions_denies() {
    let world = installed_delegated_capability_world();
    world.authorization_time.script([time(100), time(102)]);
    world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
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
            let host_custody = phase.test_host_request();
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
            let host_custody = phase.test_host_request();
            authenticated_principal(&(phase).test_host_request(), &world, &request)
        })
        .expect("the fixture read owns its installed host call");
    let capability = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
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
            let host_custody = phase.test_host_request();
            world
                .selected_product(&(phase).test_host_request())
                .admit_governed_application_query(
                    &phase,
                    &query,
                    &access,
                    capability,
                    ApplicationQueryParameterSet::<GovernedAccountOmissionQuery>::new(),
                    crate::domain_computation::primary_graph::WorthQueryProductQueryControls::new(
                        NonZeroUsize::new(1).unwrap(),
                        NonZeroUsize::new(256).unwrap(),
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
            WorthQueryOperationAuthorizationDenialKind::CapabilityGrantMissing,
        )
    );
    let authorization = denial
        .authorization_denial()
        .expect("governed readmission must retain its exact authorization denial");
    assert!(authorization.identity().is_some());
    assert_eq!(
        authorization.causes(),
        [WorthQueryOperationAuthorizationDenialKind::CapabilityGrantMissing]
    );
}
