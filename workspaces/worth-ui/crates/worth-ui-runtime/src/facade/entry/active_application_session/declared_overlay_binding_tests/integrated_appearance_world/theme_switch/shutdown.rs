use super::*;
use crate::facade::entry::portal_dismissal::UiPortalDismissalPublicationOutcome;
use crate::runtime::portal::{UiPortalIdentity, UiPortalLifecyclePosture};

pub(super) fn with_portal_exit(
    mut world: World,
    parent: UiPortalIdentity,
    child: UiPortalIdentity,
) {
    world.host.push_native_display_settled_without_effects();
    world.host.push_native_display_settled_without_effects();
    assert!(matches!(
        world
            .session
            .publish_anchor_loss_portal_dismissal(parent, 700),
        UiPortalDismissalPublicationOutcome::Published(_)
    ));
    let portal = world.session.portal.as_ref().unwrap();
    assert_eq!(portal.posture(parent), UiPortalLifecyclePosture::Closing);
    assert_eq!(portal.posture(child), UiPortalLifecyclePosture::Closed);
    assert_eq!(
        world
            .session
            .motion
            .as_ref()
            .unwrap()
            .census()
            .exit_retentions(),
        1
    );
    clean(world, 1);
}

pub(super) fn clean(world: World, cancelled_exits: u16) {
    let receipt = world.session.shutdown();
    assert_eq!(receipt.portal_final_active_records(), 0);
    assert_eq!(receipt.motion_cancelled_exit_retentions(), cancelled_exits);
    assert!(receipt.motion_final_census_is_zero());
    assert!(receipt.runtime_service_resource_census().is_empty());
    assert!(receipt.intent_resource_census().is_empty());
    assert!(receipt.rebind().is_empty());
    assert!(receipt.mounted_presentation().query_close_complete());
    assert!(receipt
        .mounted_presentation()
        .query_transition_trace_complete());
    assert!(receipt
        .mounted_presentation()
        .query_semantic_frontier_trace_complete());
    assert!(matches!(
        receipt.host_session_release(),
        Some(UiHostSessionReleaseOutcome::Released(_))
    ));
    assert_eq!(world.host.native_in_flight_count(), 0);
    assert_eq!(world.host.pending_presentation_count(), 0);
    assert_eq!(world.host.pending_observation_batch_count(), 0);
}
