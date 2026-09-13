use super::*;

#[test]
fn rejected_portal_consequence_cannot_publish_false_lifecycle_truth() {
    let (application, facts, recorder) =
        build_open_portal_application(UiHeadlessRecorderCapacity::new(8, 1, 4_096));
    let mut world = AdmissionWorld::launch_application_on_declared_surface(
        application,
        facts,
        "visual.identity.surface.main",
        2,
        [18, 20],
    );
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
        worth_ui_host_contract::UiMountedPortalInputShielding::ModalSurface
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
