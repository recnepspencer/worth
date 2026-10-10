use super::super::super::application_attempt::authenticated_principal;
use super::super::super::fixture::CapabilityAction;
use super::super::capability_progression::time;
use crate::domain_computation::primary_graph::WorthQueryOperationAuthorizationDenialKind;

#[test]
fn approved_receipt_rejects_another_elevation_and_a_narrowed_amount() {
    let (world, request, approved) = super::approval_transition::exact_approved_world();
    world.authorization_time.script([time(100), time(100)]);
    let principal = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            authenticated_principal(&host_custody, &world, &request)
        })
        .expect("the fixture read owns its installed host call");
    let capability = super::installed_capability(&world);
    let wrong_elevation = super::elevated_input(Some("elevation-1"));
    let mut narrowed_amount = super::elevated_input(Some("elevation-2"));
    narrowed_amount.amount = 49;

    for input in [wrong_elevation, narrowed_amount] {
        let denial = world
            .application
            .with_host_advancement(|phase| {
                let host_custody = phase.test_host_request();
                world
                    .selected_product(&host_custody)
                    .admit_approved_elevation_access(
                        &host_custody.phase_for_application(&world.application),
                        &approved,
                        &principal,
                        &capability,
                        input,
                        &request,
                    )
                    .err()
                    .expect("active use must remain inside the exact requested upper bound")
            })
            .expect("the fixture read owns its installed host call");
        assert_eq!(
            denial.kind(),
            WorthQueryOperationAuthorizationDenialKind::ElevationApprovalRejected
        );
    }
}

#[test]
fn approved_receipt_from_another_runtime_cannot_open_active_use() {
    let (_source, _source_request, foreign) = super::approval_transition::exact_approved_world();
    let (world, request, _local) = super::approval_transition::exact_approved_world();
    world.authorization_time.script([time(100)]);
    let principal = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            authenticated_principal(&host_custody, &world, &request)
        })
        .expect("the fixture read owns its installed host call");
    let capability = super::installed_capability(&world);

    let denial = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            world
                .selected_product(&host_custody)
                .admit_approved_elevation_access(
                    &host_custody.phase_for_application(&world.application),
                    &foreign,
                    &principal,
                    &capability,
                    super::elevated_input(Some("elevation-2")),
                    &request,
                )
                .err()
                .expect("foreign runtime lifecycle authority must not compose locally")
        })
        .expect("the fixture read owns its installed host call");
    assert_eq!(
        denial.kind(),
        WorthQueryOperationAuthorizationDenialKind::ElevationApprovalRejected
    );
}

#[test]
fn approved_touch_elevation_cannot_authorize_a_disbursement_request() {
    let (world, request, approved) = super::approval_transition::exact_approved_world();
    world.authorization_time.script([time(100)]);
    let principal = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            authenticated_principal(&host_custody, &world, &request)
        })
        .expect("the fixture read owns its installed host call");
    let capability = super::installed_capability(&world);
    let mut disbursement = super::elevated_input(Some("elevation-2"));
    disbursement.action = CapabilityAction::Disburse;

    let denial =
        world
            .application
            .with_host_advancement(|phase| {
                let host_custody = phase.test_host_request();
                world
            .selected_product(&host_custody)
            .admit_approved_elevation_access(
                &host_custody.phase_for_application(&world.application),
                &approved,
                &principal,
                &capability,
                disbursement,
                &request,
            )
            .err()
            .expect("the touch upper bound must reject a disbursement before operation authority")
            })
            .expect("the fixture read owns its installed host call");

    assert_eq!(
        denial.kind(),
        WorthQueryOperationAuthorizationDenialKind::CapabilityProjectionRejected
    );
}

#[test]
fn missing_direct_elevation_resource_denies_active_admission() {
    let (world, request, approved) = super::approval_transition::exact_approved_world();
    world.authorization_time.script([time(100)]);
    let principal = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            authenticated_principal(&host_custody, &world, &request)
        })
        .expect("the fixture read owns its installed host call");
    world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            let borrowed_execution = host_custody.borrowed_request_for_test();
            let execution = &borrowed_execution;
            super::mutation::replace_elevation_resource(
                &host_custody,
                execution,
                &world,
                "elevation-2",
                None,
            )
        })
        .expect("the fixture read owns its installed host call");

    let denial = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            super::admit(
                &host_custody,
                &world,
                &approved,
                &principal,
                &request,
                Some("elevation-2"),
            )
            .err()
            .expect("active authority requires the exact direct elevation resource")
        })
        .expect("the fixture read owns its installed host call");

    assert_eq!(
        denial.kind(),
        WorthQueryOperationAuthorizationDenialKind::ElevationInactive
    );
}

#[test]
fn additional_foreign_elevation_resource_denies_fresh_active_admission() {
    let (world, request, approved) = super::approval_transition::exact_approved_world();
    world.authorization_time.script([time(100)]);
    let principal = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            authenticated_principal(&host_custody, &world, &request)
        })
        .expect("the fixture read owns its installed host call");
    world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            super::mutation::add_elevation_resource(
                &host_custody,
                &host_custody.borrowed_request_for_test(),
                &world,
                "elevation-2",
                "account-2",
            )
        })
        .expect("the fixture read owns its installed host call");

    let denial = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            super::admit(
                &host_custody,
                &world,
                &approved,
                &principal,
                &request,
                Some("elevation-2"),
            )
            .err()
            .expect("active authority requires the complete resource adjacency to remain exact")
        })
        .expect("the fixture read owns its installed host call");

    assert_eq!(
        denial.kind(),
        WorthQueryOperationAuthorizationDenialKind::ElevationInactive
    );
}

#[test]
fn retargeted_direct_elevation_resource_stales_admitted_authority() {
    let (world, request, approved) = super::approval_transition::exact_approved_world();
    world.authorization_time.script([time(100), time(100)]);
    let principal = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            authenticated_principal(&host_custody, &world, &request)
        })
        .expect("the fixture read owns its installed host call");
    let access = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            super::admit(
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
            let host_custody = phase.test_host_request();
            let borrowed_execution = host_custody.borrowed_request_for_test();
            let execution = &borrowed_execution;
            super::mutation::replace_elevation_resource(
                &host_custody,
                execution,
                &world,
                "elevation-2",
                Some("account-2"),
            )
        })
        .expect("the fixture read owns its installed host call");
    let operation = world
        .application
        .installed_schema()
        .installed_operation(
            super::super::super::fixture::ElevatedCapabilityTouchOperation::reference(),
        )
        .unwrap();

    let denial = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            world
                .application
                .authorize_capability_operation(
                    &host_custody.phase_for_application(&world.application),
                    access,
                    &operation,
                    Default::default(),
                )
                .err()
                .expect("retargeting the direct resource must stale admitted authority")
        })
        .expect("the fixture read owns its installed host call");

    assert_eq!(
        denial.kind(),
        WorthQueryOperationAuthorizationDenialKind::StaleAuthorization
    );
}

#[test]
fn additional_foreign_elevation_resource_stales_admitted_authority() {
    let (world, request, approved) = super::approval_transition::exact_approved_world();
    world.authorization_time.script([time(100), time(100)]);
    let principal = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            authenticated_principal(&host_custody, &world, &request)
        })
        .expect("the fixture read owns its installed host call");
    let access = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            super::admit(
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
            let host_custody = phase.test_host_request();
            super::mutation::add_elevation_resource(
                &host_custody,
                &host_custody.borrowed_request_for_test(),
                &world,
                "elevation-2",
                "account-2",
            )
        })
        .expect("the fixture read owns its installed host call");
    let operation = world
        .application
        .installed_schema()
        .installed_operation(
            super::super::super::fixture::ElevatedCapabilityTouchOperation::reference(),
        )
        .unwrap();

    let denial = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            world
                .application
                .authorize_capability_operation(
                    &host_custody.phase_for_application(&world.application),
                    access,
                    &operation,
                    Default::default(),
                )
                .err()
                .expect("an additive resource edge must stale the exact retained adjacency")
        })
        .expect("the fixture read owns its installed host call");

    assert_eq!(
        denial.kind(),
        WorthQueryOperationAuthorizationDenialKind::StaleAuthorization
    );
}
