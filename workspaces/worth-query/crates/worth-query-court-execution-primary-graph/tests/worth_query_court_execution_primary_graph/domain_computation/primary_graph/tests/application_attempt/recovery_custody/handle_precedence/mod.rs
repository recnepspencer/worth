//! Handle expiry precedes the same foreign installed seal at issue and use.
use crate::domain_computation::primary_graph::tests::application_attempt::recovery_custody::*;
use worth_query_execution::facade::domain_computation::WorthQueryOperationAuthorizationDenialKind as Authorization;
use worth_query_execution::facade::domain_computation::WorthQueryOperationAuthorizationRefusalKind as Refusal;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::TouchAccountOperation;

pub fn check(redispatch: bool) {
    let world = installed_authorization_world(
        true,
        worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system(),
    );
    let now = std::time::SystemTime::now();
    world.authorization_time.hold(now);
    let (handle, mut admission, _) = committed_handle(&world);
    let authority = world
        .application
        .admit_recovery_effect_authority(&handle, &admission)
        .unwrap();
    let donor = world
        .application
        .installed_schema()
        .installed_operation(TouchAccountOperation::reference())
        .unwrap();
    admission.forge_installed_seal_for_test(donor.installed_view());
    assert_eq!(
        world
            .application
            .admit_recovery_effect_authority(&handle, &admission)
            .unwrap_err()
            .kind(),
        WorthQueryRecoveryHandleDenialKind::AdmissionAuthorization(Authorization::Refused(
            Refusal::StaleInstalledOperation
        ))
    );
    world
        .authorization_time
        .hold(now + Duration::from_secs(3_601));
    if redispatch {
        world
            .application
            .with_host_advancement(|phase| {
                let Err(denial) = world
                    .application
                    .redispatch_admitted_external_effect(&phase, &handle, &authority, &admission)
                else {
                    panic!("the expired handle must refuse redispatch");
                };
                assert!(
                    matches!(
                        denial,
                        WorthQueryExternalRedispatchDenial::FreshAuthority(
                            WorthQueryRecoveryHandleDenialKind::Expired
                        )
                    ),
                    "{denial:?}"
                );
            })
            .unwrap();
    } else {
        assert_eq!(
            world
                .application
                .admit_recovery_effect_authority(&handle, &admission)
                .unwrap_err()
                .kind(),
            WorthQueryRecoveryHandleDenialKind::Expired
        );
    }
}

#[test]
fn entry_custody_expired_handle_precedes_foreign_seal_at_recovery_authority_issuance() {
    check(false);
}
#[test]
fn entry_custody_expired_handle_precedes_foreign_seal_at_redispatch() {
    check(true);
}
