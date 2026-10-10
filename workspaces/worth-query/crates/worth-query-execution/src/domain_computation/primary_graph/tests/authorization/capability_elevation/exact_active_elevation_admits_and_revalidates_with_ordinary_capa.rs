//! exact active elevation admits and revalidates with ordinary capability authority.
use super::*;

#[test]
fn exact_active_elevation_admits_and_revalidates_with_ordinary_capability_authority() {
    crate::domain_computation::primary_graph::with_test_advancement(|host_custody| {
        let (world, request, approved) = approval_transition::exact_approved_world(&host_custody);
        assert_eq!(
            world
                .application
                .capability_plan_compilation_evidence()
                .elevation_lifecycle_operation_count(),
            4
        );
        let (_, _, request_role) = world
        .application
        .authorization
        .elevation_lifecycle_operation::<IdentityExecutionSchema, RequestCapabilityElevationOperation>(
            "RequestCapabilityElevationOperation",
            <RequestElevationInput as worth_query_declaration::facade::portable_identity::WorthQueryPortableType>::PORTABLE_TYPE_IDENTITY.as_str(),
        )
        .expect("the request marker must match installed lifecycle identity")
        .expect("the request operation must have one lifecycle owner");
        assert_eq!(format!("{request_role:?}"), "Request");
        assert!(world
        .application
        .authorization
        .elevation_lifecycle_operation::<
            IdentityExecutionSchema,
            AliasedRequestCapabilityElevationOperation,
        >(
            "RequestCapabilityElevationOperation",
            <RequestElevationInput as worth_query_declaration::facade::portable_identity::WorthQueryPortableType>::PORTABLE_TYPE_IDENTITY.as_str(),
        )
        .is_err());
        world.authorization_time.script([time(100), time(100)]);
        let principal = authenticated_principal(
            &(host_custody.phase_for_application(&world.application)).test_host_request(),
            &world,
            &request,
        );
        let access = admit(
            &(host_custody.phase_for_application(&world.application)).test_host_request(),
            &world,
            &approved,
            &principal,
            &request,
            Some("elevation-2"),
        )
        .unwrap();

        assert_eq!(access.authorization_decision_fact_count(), 2);
        let operation = world
            .application
            .installed_schema()
            .installed_operation(ElevatedCapabilityTouchOperation::reference())
            .unwrap();
        world
            .application
            .authorize_capability_operation(
                &host_custody.phase_for_application(&world.application),
                access,
                &operation,
                Default::default(),
            )
            .expect("the exact active elevation must survive fresh operation re-admission");
    })
}
