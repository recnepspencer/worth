//! exact active elevation admits and revalidates with ordinary capability authority.
use crate::domain_computation::primary_graph::tests::authorization::capability_elevation::*;

#[test]
fn exact_active_elevation_admits_and_revalidates_with_ordinary_capability_authority() {
    let (world, request, approved) = crate::domain_computation::primary_graph::tests::authorization::capability_elevation::approval_transition::exact_approved_world();
    assert_eq!(
        world
            .application
            .capability_plan_compilation_evidence()
            .elevation_lifecycle_operation_count(),
        4
    );
    let (_, _, request_role) = (*world
        .application
        .authorization_for_test())
        .elevation_lifecycle_operation_for_test::<IdentityExecutionSchema, RequestCapabilityElevationOperation>(
            "RequestCapabilityElevationOperation",
            <RequestElevationInput as worth_query_declaration::facade::portable_identity::WorthQueryPortableType>::PORTABLE_TYPE_IDENTITY.as_str(),
        )
        .expect("the request marker must match installed lifecycle identity")
        .expect("the request operation must have one lifecycle owner");
    assert_eq!(format!("{request_role:?}"), "Request");
    assert!((*world
        .application
        .authorization_for_test())
        .elevation_lifecycle_operation_for_test::<
            IdentityExecutionSchema,
            AliasedRequestCapabilityElevationOperation,
        >(
            "RequestCapabilityElevationOperation",
            <RequestElevationInput as worth_query_declaration::facade::portable_identity::WorthQueryPortableType>::PORTABLE_TYPE_IDENTITY.as_str(),
        )
        .is_err());
    world.authorization_time.script([time(100), time(100)]);
    let principal = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request_for_test();
            authenticated_principal(
                &(host_custody.phase_for_application_for_test(&world.application)).test_host_request_for_test(),
                &world,
                &request,
            )
        })
        .expect("the fixture read owns its installed host call");
    let access = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request_for_test();
            admit(
                &(host_custody.phase_for_application_for_test(&world.application)).test_host_request_for_test(),
                &world,
                &approved,
                &principal,
                &request,
                Some("elevation-2"),
            )
            .unwrap()
        })
        .expect("the fixture read owns its installed host call");

    assert_eq!(access.authorization_decision_fact_count(), 2);
    let operation = world
        .application
        .installed_schema()
        .installed_operation(ElevatedCapabilityTouchOperation::reference())
        .unwrap();
    world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request_for_test();
            world
                .application
                .authorize_capability_operation(
                    &host_custody.phase_for_application_for_test(&world.application),
                    access,
                    &operation,
                    Default::default(),
                )
                .expect("the exact active elevation must survive fresh operation re-admission")
        })
        .expect("the fixture read owns its installed host call");
}
