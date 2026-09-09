use super::fixture::*;
use crate::certification_support::ScriptedSurfaceCompletion;
use crate::mounting::UiMountedMotionSampleSettlement;
use crate::runtime::motion::UiMotionTargetIdentity;
use worth_ui_host_contract::*;

#[test]
fn pending_motion_denies_semantic_mount_without_changing_accepted_truth() {
    let (mut session, host, surface, command) = mounted();
    let original = session
        .mounted
        .current_presentation_for_surface(surface)
        .unwrap();
    let target =
        UiMotionTargetIdentity::from_family_owner(surface, command.mounted_instance(), 841);
    install(&mut session, target, original, 841, false, None);
    host.push_in_flight(
        vec![ScriptedSurfaceCompletion::Presented(completion(2, true))],
        UiHostSurfaceCancellationOutcome::CancelledBeforeEffects,
    );
    let prepared = session.mounted.prepare_motion_tick(1, original).unwrap();
    assert!(matches!(
        session
            .mounted
            .present_prepared_motion_tick(&session.host_session, prepared, original),
        UiMountedMotionSampleSettlement::Deferred
    ));
    let graph = session
        .mounted
        .current_mounted_identity_basis(command.mounted_instance())
        .unwrap()
        .graph_node_identity();
    let node = session.mounted_graph_node(graph).unwrap();
    let instances = session.mounted_instances_for(node).unwrap();
    assert!(matches!(
        session.mount_instance(node, surface),
        Err(crate::mounting::UiMountedIdentityDenial::PresentationInFlight)
    ));
    assert_eq!(session.mounted_instances_for(node).unwrap(), instances);
    assert_eq!(
        session.mounted.current_presentation_for_surface(surface),
        Some(original)
    );
    assert_eq!(
        session
            .mounted
            .accepted_motion_for_command(original, command)
            .unwrap(),
        None
    );
    assert!(matches!(
        session
            .mounted
            .complete_motion_sample_presentation(&session.host_session),
        Some(UiMountedMotionSampleSettlement::Committed(_))
    ));
    let current = session
        .mounted
        .current_presentation_for_surface(surface)
        .unwrap();
    assert_eq!(current.frame(), original.frame());
    assert_ne!(current, original);
    let accepted = session
        .mounted
        .accepted_motion_for_command(current, command)
        .unwrap()
        .unwrap();
    assert_eq!(accepted.presentation_basis(), current);
    assert_eq!(accepted.opacity_units(), 65_535);
    assert!(session
        .mounted
        .accepted_motion_for_command(original, command)
        .is_err());
    session.mount_instance(node, surface).unwrap();
    assert_eq!(
        session.mounted_instances_for(node).unwrap().len(),
        instances.len() + 1
    );
    let _ = session.shutdown();
}
