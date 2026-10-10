use std::num::NonZeroUsize;
use std::time::{Duration, UNIX_EPOCH};

use super::{
    WorthQueryGovernedTemporalOperationAuthorization, WorthQueryGovernedTemporalQueryAuthorization,
    WorthQueryTemporalOperationAuthorization, WorthQueryTemporalQueryAuthorization,
};
use crate::domain_computation::primary_graph::tests::fixture::capability::{
    CapabilityAction, CapabilityDisclosure, CapabilityPurpose, CapabilityTouchInput,
    CapabilityTouchOperation, TouchAccountCapability,
};
use crate::domain_computation::primary_graph::tests::{
    application_attempt::{authenticated_principal, resolved_account},
    fixture::{installed_capability_authorization_world, live_scope, GovernedAccountOmissionQuery},
};
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationQueryAccessContext, WorthQueryApplicationQueryControls,
};

#[test]
fn governed_temporal_operation_uses_real_capability_progression() {
    let world = installed_capability_authorization_world();
    world.authorization_time.script([
        UNIX_EPOCH + Duration::from_secs(100),
        UNIX_EPOCH + Duration::from_secs(100),
    ]);
    let request = live_scope();
    world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            let borrowed_execution = host_custody.borrowed_request_for_test();
            let execution = &borrowed_execution;
            let principal = authenticated_principal(
                &(host_custody.phase_for_application(&world.application)).test_host_request(),
                &world,
                &request,
            );
            let account = resolved_account(
                &(host_custody.phase_for_application(&world.application)).test_host_request(),
                &world,
                "open",
                &request,
            );
            let operation = world
                .application
                .installed_schema()
                .installed_operation(CapabilityTouchOperation::reference())
                .unwrap();
            let capability = world
                .application
                .installed_schema()
                .capability(
                    TouchAccountCapability::reference(),
                    CapabilityTouchOperation::reference(),
                )
                .unwrap();
            let input = admitted_input();
            let authorization = WorthQueryGovernedTemporalOperationAuthorization::new(capability);
            let selected = world.selected_product(
                &(host_custody.phase_for_application(&world.application)).test_host_request(),
            );
            let admission = authorization
                .authorize(
                    execution,
                    &selected,
                    &principal,
                    &account,
                    &operation,
                    &input,
                    Default::default(),
                    &request,
                )
                .expect("current capability must admit the temporal operation");
            assert_eq!(admission.operation(), "CapabilityTouchOperation");
            assert!(admission.authorization_requirement_count() > 0);
        })
        .expect("the fixture block owns its installed host call")
}

#[test]
fn governed_temporal_reconstruction_uses_real_capability_progression() {
    let world = installed_capability_authorization_world();
    world.authorization_time.script([
        UNIX_EPOCH + Duration::from_secs(100),
        UNIX_EPOCH + Duration::from_secs(100),
    ]);
    let request = live_scope();
    world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            let borrowed_execution = host_custody.borrowed_request_for_test();
            let execution = &borrowed_execution;
            let principal = authenticated_principal(
                &(host_custody.phase_for_application(&world.application)).test_host_request(),
                &world,
                &request,
            );
            let account = resolved_account(
                &(host_custody.phase_for_application(&world.application)).test_host_request(),
                &world,
                "open",
                &request,
            );
            let query = world
                .application
                .installed_schema()
                .certification_query(GovernedAccountOmissionQuery::reference())
                .unwrap();
            let capability = world
                .application
                .installed_schema()
                .capability(
                    TouchAccountCapability::reference(),
                    CapabilityTouchOperation::reference(),
                )
                .unwrap();
            let authorization =
                WorthQueryGovernedTemporalQueryAuthorization::new(capability, admitted_input());
            let access = WorthQueryApplicationQueryAccessContext::new(&principal, &account);
            let selected = world.selected_product(
                &(host_custody.phase_for_application(&world.application)).test_host_request(),
            );
            let (_, product, application_basis) = selected.into_parts();
            let plan = authorization
        .admit(execution,
            &world.application,
            &query,
            &access,
            worth_query_declaration::facade::application_query::ApplicationQueryParameterSet::new(),
            WorthQueryApplicationQueryControls::product_one_shot(
                product,
                application_basis,
                NonZeroUsize::new(1).unwrap(),
                NonZeroUsize::new(256).unwrap(),
                &request,
            ),
        )
        .expect("current capability must admit temporal reconstruction");
            assert!(plan.graph_work_capability_identity().is_some());
        })
        .expect("the fixture block owns its installed host call")
}

fn admitted_input() -> CapabilityTouchInput {
    CapabilityTouchInput {
        account: "account-1".to_owned(),
        action: CapabilityAction::Touch,
        purpose: CapabilityPurpose::AccountMaintenance,
        disclosure: CapabilityDisclosure::AccountActivity,
        related_account: "account-2".to_owned(),
        request_record: "selected-request".to_owned(),
        prior_record: "selected-prior".to_owned(),
        amount: 50,
        caller_time: 100,
    }
}
