use worth_ui_host_contract::{
    UiHostPresentationCostInput, UiHostPresentationCostReport, UiHostSurfaceCancellationOutcome,
    UiHostSurfacePresentationMode, UiMountedCompletedEffects, UiMountedEffectFamily,
    UiMountedSurfacePresentationCompletion,
};
use worth_ui_runtime::certification_support::ScriptedSurfaceCompletion;
use worth_ui_test_support::{
    UiPortalDismissalCertificationOutcome, UiPortalExitTerminalCertificationOutcome,
    WorthUiMotionPresentationCertificationExt, WorthUiPortalRuntimeCertificationExt,
    WorthUiServiceProposalCertificationExt,
};

use super::motion_sampling::{
    assert_motion_tick_applied, launch_scripted_motion_world, motion_tick_batch,
    scripted_motion_host,
};
use crate::intent::admission::phase3::world::AdmissionWorld;

#[test]
fn exit_motion_retains_closing_overlay_until_terminal_portal_publication() {
    let host = scripted_motion_host();
    for _ in 0..6 {
        host.push_presented();
    }
    let mut world = launch_scripted_motion_world(host);

    terminalize_portal_exit_motion(&mut world);
    assert_retention_census(&world.session);
    let closing = world.session.inspect_portal_runtime_for_certification();
    assert_eq!(closing.active_portals(), 1);
    assert_eq!(closing.closing_portals(), 1);
    assert_eq!(
        world
            .session
            .progress_portal_exit_terminal_for_certification(113),
        UiPortalExitTerminalCertificationOutcome::Published
    );

    let closed = world.session.inspect_portal_runtime_for_certification();
    assert_eq!(closed.active_portals(), 0);
    assert_eq!(closed.closing_portals(), 0);
    assert_retention_census(&world.session);
    assert!(world
        .session
        .inspect_service_proposals_for_certification()
        .is_zero());
    let shutdown = world.session.shutdown();
    assert_eq!(shutdown.motion_cancelled_exit_retentions(), 0);
    assert!(shutdown.motion_final_census_is_zero());
}

#[test]
fn shutdown_cancels_in_flight_terminal_portal_proposal_without_motion_owner_leak() {
    let host = scripted_motion_host();
    for _ in 0..5 {
        host.push_presented();
    }
    host.push_in_flight(
        vec![ScriptedSurfaceCompletion::Presented(
            UiMountedSurfacePresentationCompletion::new(
                UiHostSurfacePresentationMode::RecordOnly,
                worth_ui_host_contract::UiHostPresentationEpoch::issued_by_host(7),
                UiMountedCompletedEffects::new(vec![UiMountedEffectFamily::RecordedProjection]),
                UiHostPresentationCostReport::from_adapter(UiHostPresentationCostInput {
                    presented_surfaces: 1,
                    ..Default::default()
                }),
            ),
        )],
        UiHostSurfaceCancellationOutcome::CancelledBeforeEffects,
    );
    let mut world = launch_scripted_motion_world(host);

    terminalize_portal_exit_motion(&mut world);
    assert_eq!(
        world
            .session
            .progress_portal_exit_terminal_for_certification(113),
        UiPortalExitTerminalCertificationOutcome::AwaitingPhysical
    );
    assert_retention_census(&world.session);
    assert!(world
        .session
        .inspect_portal_runtime_for_certification()
        .pending_track_coordinated(),);
    assert_eq!(
        world
            .session
            .inspect_service_proposals_for_certification()
            .entries(),
        [
            ("proposals", 1),
            ("occupancy_leases", 3),
            ("cancellation_records", 1),
            ("stage_receipts", 5),
            ("live_occupancies", 3),
            ("live_cancellations", 1),
        ],
        "terminal closure compiles Portal, Focus, and its one Scroll reveal without a second Motion owner"
    );

    let shutdown = world.session.shutdown();
    assert_eq!(shutdown.portal_final_active_records(), 0);
    assert_eq!(shutdown.motion_terminated_active_tracks(), 0);
    assert_eq!(shutdown.motion_cancelled_exit_retentions(), 1);
    assert!(shutdown.motion_final_census_is_zero());
}

pub(super) fn terminalize_portal_exit_motion(world: &mut AdmissionWorld) {
    assert_eq!(
        world
            .session
            .publish_escape_portal_dismissal_for_certification(41),
        UiPortalDismissalCertificationOutcome::Published
    );
    assert_eq!(
        world
            .session
            .inspect_portal_runtime_for_certification()
            .closing_portals(),
        1
    );
    let first_presentation = world
        .session
        .inspect_motion_presentation_for_certification()
        .presentation()
        .expect("retained exit has current presentation");
    assert_motion_tick_applied(
        world
            .session
            .admit_host_interaction_batch(motion_tick_batch(
                &world.session,
                first_presentation,
                3,
                1,
            )),
    );
    let second_presentation = world
        .session
        .inspect_motion_presentation_for_certification()
        .presentation()
        .expect("first sample advances retained presentation");
    assert_motion_tick_applied(
        world
            .session
            .admit_host_interaction_batch(motion_tick_batch(
                &world.session,
                second_presentation,
                4,
                112,
            )),
    );
    assert_eq!(
        world
            .session
            .inspect_motion_presentation_for_certification()
            .active_tracks(),
        0
    );
}

pub(super) fn assert_retention_census(
    session: &worth_ui::facade::app::WorthUiActiveApplicationSession,
) {
    let portal = session.inspect_portal_runtime_for_certification();
    let census = session.runtime_service_resource_census();
    assert_eq!(
        portal.portal_exit_retentions(),
        census.portal_exit_retentions(),
        "Portal receipt rows equal coordinator rows"
    );
    assert_eq!(
        usize::from(census.motion_exit_retentions()),
        census.portal_exit_retentions(),
        "Motion exit-retention rows equal coordinator rows"
    );
    assert_eq!(
        portal.closing_portals(),
        portal.portal_exit_retentions(),
        "only receipt-backed Portal rows remain Closing"
    );
    assert!(
        portal.pending_track_coordinated(),
        "a pending terminal retains its coordinator row"
    );
}
