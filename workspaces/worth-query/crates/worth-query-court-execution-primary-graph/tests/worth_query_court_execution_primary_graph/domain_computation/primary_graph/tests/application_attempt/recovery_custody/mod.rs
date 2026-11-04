//! Recovery's outer handle custody and interrupted admission lifetime.

pub use std::time::Duration;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::TouchAccountOperation;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::recovery_custody::committed_handle;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::world_installation::installed_authorization_world;

pub use crate::process::domain_computation::primary_graph::tests::application_attempt::*;
pub use worth_query_execution::facade::domain_computation::WorthQueryRecoveryHandle;
pub use worth_query_execution::facade::domain_computation::WorthQueryRecoveryHandleDenialKind;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::Account;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::authorization_world_installation::AuthorizationWorld;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::ExactStatusRetentionInput;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::ExactStatusRetentionOperation;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::IdentityExecutionSchema;
pub use worth_query_execution::facade::domain_computation::WorthQueryAdmittedApplicationOperation;
pub use worth_query_execution::facade::domain_computation::WorthQueryExternalRedispatchDenial;
pub use worth_query_execution::facade::domain_computation::WorthQueryOperationAuthorizationDenialKind;
pub use worth_query_admission::facade::authenticated_principal::WorthQueryCancellationSource;
pub use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
#[test]
fn expired_redispatch_handle_precedes_cancelled_admission() {
    let world = installed_authorization_world(
        true,
        worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system(),
    );
    let now = std::time::SystemTime::now();
    world.authorization_time.hold(now);
    let (handle, admission, cancellation) = committed_handle(&world);
    let authority = world
        .application
        .admit_recovery_effect_authority(&handle, &admission)
        .expect("fresh handle and genuine admission issue authority");
    cancellation.cancel();
    assert_eq!(
        admission.validate_current_authority().unwrap_err().kind(),
        WorthQueryOperationAuthorizationDenialKind::Stopped(Err(
            worth_query_execution::facade::domain_computation::WorthQueryProviderSessionControlStopKind::Cancelled
        ))
    );
    world
        .authorization_time
        .hold(now + Duration::from_secs(3_601));
    let denial = world
        .application
        .with_host_advancement(|phase| {
            let Err(denial) = world
                .application
                .redispatch_admitted_external_effect(&phase, &handle, &authority, &admission)
            else {
                panic!("the expired handle refuses redispatch");
            };
            denial
        })
        .unwrap();
    assert!(
        matches!(
            denial,
            WorthQueryExternalRedispatchDenial::FreshAuthority(
                WorthQueryRecoveryHandleDenialKind::Expired
            )
        ),
        "outer handle custody must win: {denial:?}"
    );
}

#[path = "handle_precedence/mod.rs"]
pub mod handle_precedence;
#[path = "issuance/mod.rs"]
pub mod issuance;
