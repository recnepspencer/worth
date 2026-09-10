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
    update_grant_field(
        &world,
        "capability-parent",
        field(&world, CapabilityNotAfterField::reference()),
        worth_foundational::facade::AspectValue::UInt64(101),
    );
    update_grant_field(
        &world,
        "capability-child",
        field(&world, CapabilityNotAfterField::reference()),
        worth_foundational::facade::AspectValue::UInt64(101),
    );
    let request = live_scope();
    let principal = authenticated_principal(&world, &request);
    let capability = admit_touch_account_capability(&world, &principal, &request)
        .expect("the narrowed chain is current at the first trusted sample");
    let account = world
        .application
        .select_product_branch(world.application.product_runtime().default_branch())
        .expect("the selected product branch remains admitted")
        .resolve_entity(
            AccountIdentity::reference(),
            "account-1".to_owned(),
            &request,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap();
    let query = world
        .application
        .installed_schema()
        .application_query(GovernedAccountOmissionQuery::reference())
        .unwrap();
    let access = WorthQueryApplicationQueryAccessContext::new(&principal, &account);

    let Err(denial) = world.selected_product().admit_governed_application_query(
        &query,
        &access,
        capability,
        ApplicationQueryParameterSet::<GovernedAccountOmissionQuery>::new(),
        crate::domain_computation::primary_graph::WorthQueryProductQueryControls::new(
            NonZeroUsize::new(1).unwrap(),
            NonZeroUsize::new(256).unwrap(),
            &request,
        ),
    ) else {
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
