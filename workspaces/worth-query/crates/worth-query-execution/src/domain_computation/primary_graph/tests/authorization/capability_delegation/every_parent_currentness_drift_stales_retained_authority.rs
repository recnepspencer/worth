//! every parent currentness drift stales retained authority.
use super::*;

#[test]
fn every_parent_currentness_drift_stales_retained_authority() {
    crate::domain_computation::primary_graph::with_test_advancement(|host_custody| {
        for drift in [
            ParentDrift::Revocation,
            ParentDrift::Expiry,
            ParentDrift::Workflow,
            ParentDrift::Resource,
            ParentDrift::DelegationGrantor,
            ParentDrift::Grantee,
            ParentDrift::PolicyPath,
            ParentDrift::Substitute,
        ] {
            let world = installed_delegated_capability_world();
            world.authorization_time.script([time(100), time(100)]);
            let request = live_scope();
            let principal = authenticated_principal(
                &(host_custody.phase_for_application(&world.application)).test_host_request(),
                &world,
                &request,
            );
            let access = admitted_capability_access(
                &(host_custody.phase_for_application(&world.application)).test_host_request(),
                &world,
                &principal,
                &request,
                100,
            )
            .unwrap();
            apply_parent_drift(
                &host_custody.phase_for_application(&world.application),
                &host_custody.borrowed_request_for_test(),
                &world,
                drift,
            );
            let operation = world
                .application
                .installed_schema()
                .installed_operation(
                    super::super::super::fixture::CapabilityTouchOperation::reference(),
                )
                .unwrap();
            let Err(denial) = world.application.authorize_capability_operation(
                &host_custody.phase_for_application(&world.application),
                access,
                &operation,
                Default::default(),
            ) else {
                panic!("retained authority must stale before a parent can be refreshed");
            };
            assert_eq!(
                denial.kind(),
                WorthQueryOperationAuthorizationDenialKind::StaleAuthorization,
                "drift {drift:?} did not invalidate the composite fact"
            );
        }
    })
}
