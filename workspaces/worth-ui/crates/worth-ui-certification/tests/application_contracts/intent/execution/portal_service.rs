use worth_ui::facade::{
    intent::{
        UiIntentConsequencePublicationOutcome, UiIntentConsequenceStopReason, UiIntentDefinition,
        UiIntentExecutionDispatchOutcome, UiIntentRuntimeServiceDestination,
    },
    rebind::{UiRebindExecutionPolicy, UiRebindExecutionRequest},
};
use worth_ui_host_headless::UiHeadlessRecorderCapacity;
use worth_ui_test_support::{
    UiPortalDismissalCertificationOutcome, WorthUiFocusRuntimeCertificationExt,
    WorthUiPortalRuntimeCertificationExt, WorthUiServiceProposalCertificationExt,
};

use super::execution_deadline;
use crate::intent::{
    admission::phase3::world::AdmissionWorld,
    operability::{
        build_open_portal_application, build_open_portal_application_with_host, PrimaryIntent,
    },
};

#[path = "portal_service/application_replacement_lifecycle.rs"]
mod application_replacement_lifecycle;
#[path = "portal_service/application_replacement_recovery.rs"]
mod application_replacement_recovery;
#[path = "portal_service/content_rebind_reconstruction.rs"]
mod content_rebind_reconstruction;
#[path = "portal_service/drop_cleanup.rs"]
mod drop_cleanup;
#[path = "portal_service/exit_retention.rs"]
mod exit_retention;
#[path = "portal_service/motion_sampling.rs"]
mod motion_sampling;
#[path = "portal_service/native_anchor_loss.rs"]
mod native_anchor_loss;
#[path = "portal_service/native_duplicate_dismissal.rs"]
mod native_duplicate_dismissal;
#[path = "portal_service/native_focus_navigation.rs"]
mod native_focus_navigation;
#[path = "portal_service/native_recovery.rs"]
mod native_recovery;
#[path = "portal_service/support.rs"]
mod support;
pub(crate) use native_duplicate_dismissal::run_native_runtime_service_scenario;
use support::only_transition;

#[test]
fn application_rebind_preserves_open_portal_overlay_and_retargets_focus() {
    let (application, facts, recorder) =
        build_open_portal_application(UiHeadlessRecorderCapacity::new(8, 8, 16_384));
    let mut world =
        AdmissionWorld::launch_application_with_target(application, facts, 1, 2, [18, 20]);
    let definition = UiIntentDefinition::<PrimaryIntent>::runtime_service(
        UiIntentRuntimeServiceDestination::OpenPortal,
    );
    let admitted = world.admit_exact_definition(0, definition);
    assert!(matches!(
        world
            .session
            .dispatch_admitted_intent(admitted, execution_deadline(20)),
        UiIntentExecutionDispatchOutcome::AttemptPrepared(_)
    ));
    let handle = only_transition(&mut world)
        .into_consequence()
        .expect("completed portal intent retains its mounted consequence");
    match world.session.publish_intent_consequences(
        handle,
        UiRebindExecutionPolicy::ordinary(),
        UiRebindExecutionRequest::new(40),
    ) {
        UiIntentConsequencePublicationOutcome::Published(_) => {}
        UiIntentConsequencePublicationOutcome::Stopped(stop) => {
            panic!("portal publication stopped: {:?}", stop.reason())
        }
        UiIntentConsequencePublicationOutcome::InFlight(_) => {
            panic!("portal publication in flight")
        }
        UiIntentConsequencePublicationOutcome::Indeterminate(_) => {
            panic!("portal publication indeterminate")
        }
        UiIntentConsequencePublicationOutcome::NoConsequences(_) => {
            panic!("portal publication lost its consequence")
        }
        UiIntentConsequencePublicationOutcome::InternalDefect(_) => {
            panic!("portal publication reported an internal defect")
        }
    }
    let portal_before = world.session.inspect_portal_runtime_for_certification();
    let focus_before = world.session.inspect_focus_runtime_for_certification();
    assert_eq!(portal_before.visible_portals(), 1);
    let focus_placement_before = recorder
        .semantic_focus_placement(world.session.host_session_identity().as_u64())
        .expect("portal open reaches the real headless focus mechanic");
    recorder.drain_transcripts();

    world.rebind_application();

    assert_eq!(
        world.session.inspect_portal_runtime_for_certification(),
        portal_before,
        "ordinary rebind cannot drop or recommit semantic Portal truth"
    );
    let focus_after = world.session.inspect_focus_runtime_for_certification();
    assert_eq!(
        focus_after.current_participant(),
        focus_before.current_participant(),
        "stable focus identity survives the application successor"
    );
    assert_eq!(focus_after.pending_portal_transitions(), 0);
    let transcripts = recorder.drain_transcripts();
    assert_eq!(transcripts.len(), 1);
    let transcript = &transcripts[0];
    assert_eq!(transcript.portal_overlays().len(), 1);
    let overlay = transcript.portal_overlays()[0];
    assert!(transcript
        .nodes()
        .iter()
        .any(|node| node.mounted_instance() == overlay.owner()));
    let focus_acknowledgement = recorder
        .semantic_focus_placement(world.session.host_session_identity().as_u64())
        .expect("rebind focus retarget reaches the real headless focus mechanic");
    assert_ne!(
        focus_acknowledgement.request().identity(),
        focus_placement_before.request().identity(),
        "the rebind proof cannot reuse the open acknowledgement"
    );
    assert_eq!(
        focus_acknowledgement.request().presentation().frame(),
        transcript.frame()
    );
    assert_eq!(
        focus_acknowledgement.request().presentation().binding(),
        transcript.binding()
    );
    assert_ne!(
        focus_acknowledgement.request().target().node_receipt(),
        focus_placement_before.request().target().node_receipt(),
        "successor focus placement must consume the successor mounted receipt"
    );
    assert!(transcript.nodes().iter().any(|node| {
        node.mounted_instance() == focus_acknowledgement.request().target().mounted_instance()
    }));
    assert_eq!(
        world
            .session
            .publish_outside_portal_dismissal_for_certification(42),
        UiPortalDismissalCertificationOutcome::Published,
        "outside dismissal after rebind requires the Portal owner to carry the exact successor presentation basis"
    );
}

#[test]
fn indeterminate_portal_publication_retains_both_family_successors_until_shutdown() {
    let host = worth_ui_runtime::certification_support::ScriptedPresentationHost::default();
    host.set_capabilities(
        worth_ui_host_contract::WorthUiHostCapabilityReport::available(vec![
            worth_ui_host_contract::WorthUiHostCapability::MountedFrameRecording,
            worth_ui_host_contract::WorthUiHostCapability::ViewportObservation,
        ]),
    );
    host.push_presented();
    host.push_presentation(
        worth_ui_runtime::facade::mounted::UiHostSurfacePresentationOutcome::
            PresentationIndeterminate,
    );
    let (application, facts) = build_open_portal_application_with_host(host);
    let mut world =
        AdmissionWorld::launch_application_with_target(application, facts, 1, 2, [18, 20]);
    let definition = UiIntentDefinition::<PrimaryIntent>::runtime_service(
        UiIntentRuntimeServiceDestination::OpenPortal,
    );
    let admitted = world.admit_exact_definition(0, definition);
    assert!(matches!(
        world
            .session
            .dispatch_admitted_intent(admitted, execution_deadline(20)),
        UiIntentExecutionDispatchOutcome::AttemptPrepared(_)
    ));
    let handle = only_transition(&mut world)
        .into_consequence()
        .expect("completed portal intent retains its mounted consequence");
    let portal_before = world.session.inspect_portal_runtime_for_certification();
    let focus_before = world.session.inspect_focus_runtime_for_certification();

    let recovery = match world.session.publish_intent_consequences(
        handle,
        UiRebindExecutionPolicy::ordinary(),
        UiRebindExecutionRequest::new(40),
    ) {
        UiIntentConsequencePublicationOutcome::Indeterminate(recovery) => recovery,
        _ => panic!("the scripted production host port must report indeterminate publication"),
    };

    let (portal_pending, focus_pending, proposals_pending) =
        recovery.inspect_retained_service_family_state_for_certification();
    assert_eq!(
        portal_pending, portal_before,
        "indeterminate physical work cannot commit Portal truth"
    );
    assert_eq!(
        focus_pending.current_participant(),
        focus_before.current_participant()
    );
    assert_eq!(focus_pending.revision(), focus_before.revision());
    assert_eq!(focus_pending.pending_portal_transitions(), 1);
    assert!(!proposals_pending.is_zero());

    {
        let session = recovery.into_session_for_shutdown();
        assert_eq!(
            session.inspect_portal_runtime_for_certification(),
            portal_before
        );
        assert_eq!(
            session
                .inspect_focus_runtime_for_certification()
                .pending_portal_transitions(),
            0
        );
        assert!(session
            .inspect_service_proposals_for_certification()
            .is_zero());
    }
    let shutdown = world.session.shutdown();
    assert_eq!(shutdown.portal_final_active_records(), 0);
    assert!(shutdown.motion_final_census_is_zero());
    assert_eq!(shutdown.portal_abandoned_indeterminate_records(), 0);
    assert_eq!(
        shutdown.focus_placement().abandoned_indeterminate_request(),
        None
    );
    assert_eq!(shutdown.intent_execution().active_after(), 0);
    assert_eq!(shutdown.intent_admission().active_after(), 0);
}

#[test]
fn rejected_portal_consequence_cannot_publish_false_lifecycle_truth() {
    let (application, facts, recorder) =
        build_open_portal_application(UiHeadlessRecorderCapacity::new(8, 1, 4_096));
    let mut world =
        AdmissionWorld::launch_application_with_target(application, facts, 1, 2, [18, 20]);
    assert_eq!(recorder.observed_transcripts().len(), 1);

    let definition = UiIntentDefinition::<PrimaryIntent>::runtime_service(
        UiIntentRuntimeServiceDestination::OpenPortal,
    );
    let admitted = world.admit_exact_definition(0, definition);
    assert!(matches!(
        world
            .session
            .dispatch_admitted_intent(admitted, execution_deadline(20)),
        UiIntentExecutionDispatchOutcome::AttemptPrepared(_)
    ));
    let handle = only_transition(&mut world)
        .into_consequence()
        .expect("completed runtime service retains its declared mounted consequence");
    let before = world.session.inspect_portal_runtime_for_certification();
    let focus_before = world.session.inspect_focus_runtime_for_certification();
    assert!(world
        .session
        .inspect_service_proposals_for_certification()
        .is_zero());
    assert_eq!(before.active_portals(), 0);
    assert_eq!(before.visible_portals(), 0);
    assert_eq!(before.committed_requests(), 0);
    assert_eq!(before.revision(), 0);
    assert_eq!(focus_before.current_participant(), None);
    assert!(focus_before.participant_count() > 0);
    assert_eq!(focus_before.pending_portal_transitions(), 0);

    let recovery = match world.session.publish_intent_consequences(
        handle,
        UiRebindExecutionPolicy::ordinary(),
        UiRebindExecutionRequest::new(40),
    ) {
        UiIntentConsequencePublicationOutcome::Stopped(stop) => {
            assert!(
                matches!(
                    stop.reason(),
                    UiIntentConsequenceStopReason::HostRejectedBeforeEffects { rejection_count: 1 }
                ),
                "unexpected portal rejection: {:?}",
                stop.reason()
            );
            stop.into_recovery()
        }
        _ => panic!("the full recorder must reject the portal consequence before effects"),
    };
    let rejected = world.session.inspect_portal_runtime_for_certification();
    let focus_rejected = world.session.inspect_focus_runtime_for_certification();
    assert_eq!(rejected, before);
    assert_eq!(focus_rejected, focus_before);
    assert!(world
        .session
        .inspect_service_proposals_for_certification()
        .is_zero());

    assert_eq!(recorder.drain_transcripts().len(), 1);
    match world.session.retry_intent_consequences(
        recovery,
        UiRebindExecutionPolicy::ordinary(),
        UiRebindExecutionRequest::new(41),
    ) {
        UiIntentConsequencePublicationOutcome::Published(_) => {}
        UiIntentConsequencePublicationOutcome::Stopped(stop) => {
            panic!("consequence-only portal retry stopped: {:?}", stop.reason())
        }
        _ => panic!("the consequence-only retry must publish synchronously"),
    }

    let published = world.session.inspect_portal_runtime_for_certification();
    assert_eq!(published.active_portals(), 1);
    assert_eq!(published.open_portals(), 0);
    assert_eq!(published.visible_portals(), 1);
    assert_eq!(published.closing_portals(), 0);
    assert_eq!(published.indeterminate_portals(), 0);
    assert_eq!(published.committed_requests(), 1);
    assert_eq!(published.committed_idempotent_requests(), 0);
    assert_eq!(published.revision(), 1);
    let focus_published = world.session.inspect_focus_runtime_for_certification();
    assert_eq!(focus_published.pending_portal_transitions(), 0);
    assert_eq!(
        focus_published.participant_count(),
        focus_before.participant_count()
    );
    assert_eq!(focus_published.revision(), focus_before.revision() + 1);
    assert!(focus_published.current_participant().is_some());
    assert!(world
        .session
        .inspect_service_proposals_for_certification()
        .is_zero());
    let transcripts = recorder.observed_transcripts();
    assert_eq!(transcripts.len(), 1);
    let transcript = &transcripts[0];
    assert_eq!(transcript.portal_overlays().len(), 1);
    let overlay = transcript.portal_overlays()[0];
    assert_eq!(
        overlay.lifecycle(),
        worth_ui_host_contract::UiMountedPortalOverlayLifecyclePosture::Visible
    );
    assert_eq!(
        overlay.shielding(),
        worth_ui_host_contract::UiMountedPortalInputShielding::ContentBounds
    );
    assert!(overlay.portal_identity() != 0);
    assert!(overlay.bounds().y() >= overlay.anchor_bounds().y());
    assert_eq!(overlay.owner_receipt().mounted_instance(), overlay.owner());
    assert!(transcript
        .nodes()
        .iter()
        .any(|node| node.mounted_instance() == overlay.owner()));
    let command = worth_ui_host_contract::UiMountedPaintCommandIdentity::portal_overlay(&overlay);
    assert!(transcript
        .paint_order()
        .iter()
        .any(|order| order.command() == command));
    let focus_acknowledgement = recorder
        .semantic_focus_placement(world.session.host_session_identity().as_u64())
        .expect("published portal focus transition must reach the real headless focus mechanic");
    assert_eq!(
        focus_acknowledgement.disposition(),
        worth_ui_host_contract::UiHostFocusPlacementDisposition::Applied,
        "focus request: {:?}; transcript: {:?}",
        focus_acknowledgement.request(),
        transcript
    );
    assert!(transcript.nodes().iter().any(|node| {
        node.mounted_instance() == focus_acknowledgement.request().target().mounted_instance()
    }));
    assert_eq!(recorder.drain_transcripts().len(), 1);

    assert_eq!(
        world
            .session
            .publish_escape_portal_dismissal_for_certification(42),
        UiPortalDismissalCertificationOutcome::Published
    );
    let dismissed = world.session.inspect_portal_runtime_for_certification();
    assert_eq!(dismissed.active_portals(), 1);
    assert_eq!(dismissed.visible_portals(), 0);
    assert_eq!(dismissed.closing_portals(), 1);
    assert_eq!(dismissed.committed_requests(), 2);
    assert_eq!(dismissed.revision(), 2);
    assert!(world
        .session
        .inspect_service_proposals_for_certification()
        .is_zero());
    let dismissed_transcripts = recorder.observed_transcripts();
    assert_eq!(dismissed_transcripts.len(), 1);
    assert_eq!(dismissed_transcripts[0].portal_overlays().len(), 1);
    assert_eq!(
        dismissed_transcripts[0].portal_overlays()[0].lifecycle(),
        worth_ui_host_contract::UiMountedPortalOverlayLifecyclePosture::Closing
    );

    let shutdown = world.session.shutdown();
    assert_eq!(shutdown.intent_execution().active_after(), 0);
    assert_eq!(shutdown.intent_admission().active_after(), 0);
}
