use worth_query_host::facade::primary_graph::WorthQueryApplicationLiveControls;

use super::support::{
    activity_request, activity_world, approve_first, assert_exact_revoked_alternate_active,
    assert_resources_released, controls, revoke_exact_support, take_first_requested,
};
use crate::{
    BankApplicationLiveCloseOutcome, BankApplicationQueryDenial,
    BankEstateEmergencyAccessActivityLiveOutcome,
};

#[test]
fn queued_real_lifecycle_cause_is_cut_off_before_live_payload_and_closes() {
    let mut world = activity_world("estate-emergency-activity-live-cutoff");
    let live_controls = WorthQueryApplicationLiveControls::bounded(
        super::super::fixture::request_scope(),
        4,
        8,
        2_048,
    )
    .unwrap();
    let first_requested = take_first_requested(&mut world);
    let mut live = world
        .fixture
        .runtime
        .query(activity_request())
        .as_principal(&world.requester)
        .controls(controls(8))
        .subscribe_with_approved_elevation(&world.approved, live_controls)
        .expect("the exact approved elevation should open the live activity lane");

    approve_first(&world, first_requested);
    revoke_exact_support(&world, 153);
    let request = super::super::fixture::request_scope();
    let BankEstateEmergencyAccessActivityLiveOutcome::AuthorizationDenied(denial) =
        live.poll(&world.requester, &request)
    else {
        panic!("revoked live authority must terminate with its exact denial");
    };
    assert_eq!(
        denial.kind(),
        crate::BankAuthorizationDenialKind::StaleAuthorization
    );
    assert!(denial.contributing_cause_count() > 0);
    assert!(matches!(
        live.poll(&world.requester, &request),
        BankEstateEmergencyAccessActivityLiveOutcome::Closed
    ));
    assert_eq!(live.buffered_cause_count(), 0);
    let BankApplicationLiveCloseOutcome::Completed = live.close() else {
        panic!("the denied live lane must release its opening graph-read session");
    };
    assert_exact_revoked_alternate_active(&world);
    assert_resources_released(&world);
}

#[test]
fn failed_fresh_principal_resolution_ends_the_live_lease() {
    let mut world = activity_world("estate-emergency-activity-live-principal-cutoff");
    let foreign = activity_world("estate-emergency-activity-live-foreign-principal");
    let live_controls = WorthQueryApplicationLiveControls::bounded(
        super::super::fixture::request_scope(),
        4,
        8,
        2_048,
    )
    .unwrap();
    let first_requested = take_first_requested(&mut world);
    let mut live = world
        .fixture
        .runtime
        .query(activity_request())
        .as_principal(&world.requester)
        .controls(controls(8))
        .subscribe_with_approved_elevation(&world.approved, live_controls)
        .expect("approved activity should open before principal substitution");

    approve_first(&world, first_requested);
    let request = super::super::fixture::request_scope();
    assert!(matches!(
        live.poll(&foreign.requester, &request),
        BankEstateEmergencyAccessActivityLiveOutcome::StalePrincipal
    ));
    assert!(matches!(
        live.poll(&world.requester, &request),
        BankEstateEmergencyAccessActivityLiveOutcome::Closed
    ));
    assert_eq!(live.buffered_cause_count(), 0);
    assert!(matches!(
        live.close(),
        BankApplicationLiveCloseOutcome::Completed
    ));
    let application = world.fixture.runtime.application_runtime();
    assert_eq!(
        application
            .application_query_basis_observer()
            .observe()
            .active(),
        0
    );
    let buffers = application.result_buffer_observer().observe();
    assert_eq!(buffers.active_buffers(), 0);
    assert_eq!(buffers.retained_bytes(), 0);
}

#[test]
fn revoked_support_refuses_live_subscription_before_open() {
    let world = activity_world("estate-emergency-activity-live-open-cutoff");
    revoke_exact_support(&world, 155);
    let controls = WorthQueryApplicationLiveControls::bounded(
        super::super::fixture::request_scope(),
        4,
        8,
        2_048,
    )
    .unwrap();
    let result = world
        .fixture
        .runtime
        .query(activity_request())
        .as_principal(&world.requester)
        .controls(super::support::controls(8))
        .subscribe_with_approved_elevation(&world.approved, controls);
    let Err(BankApplicationQueryDenial::CapabilityAdmission(denial)) = result else {
        panic!("lost exact support must deny subscription before opening");
    };
    assert_eq!(
        denial.kind(),
        crate::BankAuthorizationDenialKind::StaleAuthorization
    );
    assert_eq!(
        world
            .fixture
            .runtime
            .application_runtime()
            .application_query_basis_observer()
            .observe()
            .active(),
        0
    );
}
