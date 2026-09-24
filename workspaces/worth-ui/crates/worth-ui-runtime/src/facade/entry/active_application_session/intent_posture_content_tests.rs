use super::{fixture, prepare};
use crate::certification_support::ScriptedPresentationOutcome;

#[test]
fn intent_posture_projects_text_only_for_components_with_text_authority() {
    for admits_text in [false, true] {
        let role = fixture::role();
        let mut component = fixture::component();
        if admits_text {
            component = component.with_semantic_text(
                crate::capability::ComponentSemanticTextContract::body_default(
                    crate::capability::ThemeTokenId::new(
                        super::super::support::APPEARANCE_BASE_TOKEN,
                    )
                    .unwrap(),
                    7,
                ),
            );
        }
        let (mut session, host) = fixture::session_with_component(
            &role,
            fixture::source_with_role(if admits_text { None } else { Some(&role) }, 1),
            component,
        );
        let graph = session
            .graph()
            .node_identities()
            .find(|identity| {
                session
                    .graph()
                    .lookup()
                    .graph_node(*identity)
                    .is_some_and(|node| {
                        node.value().declaration_identity().authored_semantic_name()
                            == format!("component:{}", super::super::support::APPEARANCE_NODE_A)
                    })
            })
            .expect("authored intent control");
        let (surface, _) =
            super::super::mounting_fixture::mount_graph_node(&mut session, 1_000, graph);
        if admits_text {
            session
                .admit_application_semantic_text(&[
                    crate::native_platform::UiNativeComponentSemanticTextChange::new(
                        format!("component:{}", super::super::support::APPEARANCE_NODE_A),
                        "Ready",
                    )
                    .unwrap(),
                ])
                .unwrap();
        }
        if !admits_text {
            super::close(&mut session, &role, 1, "posture-content-initial");
            session.advance_mounted_identity_frame().unwrap();
        }
        let frame = prepare(&mut session);
        publish(&mut session, &host, frame, admits_text, 1);
        let route = super::activation_route(&mut session, surface, 1);
        let posture = session
            .intent_postures
            .prepare(
                route.graph_node(),
                route.target(),
                crate::fact_contract::UiIntentPostureReference::Route(route.definition_id()),
                crate::fact_contract::UiIntentPostureKind::Denied,
            )
            .unwrap();
        use crate::facade::entry::native_intent_posture::WorthUiNativeIntentPosturePublicationOutcome as Outcome;
        let predecessor_turn = session
            .appearance_owner_snapshot
            .as_ref()
            .map(|owners| owners.turn());
        let predecessor_frame = session.mounted.current_publication().unwrap().frame();
        host.push_presentation(ScriptedPresentationOutcome::RejectedBeforeEffects(
            worth_ui_host_contract::UiHostSurfacePresentationDenial::TextAtlasPresentationDeferred,
        ));
        let outcome = session.publish_native_intent_posture(
            crate::facade::entry::WorthUiNativeIntentPosture::new(
                posture.0,
                posture.1,
                crate::fact_contract::UiIntentPostureKind::Denied,
            ),
            crate::runtime::rebind::UiRebindExecutionPolicy::ordinary(),
            crate::runtime::rebind::UiRebindExecutionRequest::new(2),
        );
        let pending = match crate::facade::entry::native_managed_rebind::normalize_managed_intent_posture(outcome) {
            crate::facade::entry::native_managed_rebind::ManagedIntentPostureNormalization::Pending(pending) => pending,
            _ => panic!("text deferral must retain the real posture transaction"),
        };
        assert_eq!(
            session
                .appearance_owner_snapshot
                .as_ref()
                .map(|owners| owners.turn()),
            predecessor_turn
        );
        assert_eq!(
            session.mounted.current_publication().unwrap().frame(),
            predecessor_frame
        );
        // Mounted readiness alone does not cover the detached rebind reservation;
        // the native shell holds fresh ingress until this transaction settles.
        if admits_text {
            host.push_native_display_presented();
        } else {
            host.push_native_display_settled_without_effects();
        }
        let receipt = match pending.complete(&mut session, 3) {
            Outcome::Published(receipt) => receipt,
            Outcome::Stopped(stop) => panic!("posture retry stopped: {stop:?}"),
            _ => panic!("exact retained posture must publish after text becomes ready"),
        };
        // Independent product oracle: a role-only control produces no text;
        // a text-admitting control publishes the denied posture label.
        match receipt.plan().content().get(graph) {
            None => assert!(!admits_text),
            Some(crate::mounting::UiMountedSemanticTextContent::Scalar(text)) => {
                assert!(admits_text);
                assert_eq!(text.posture().as_ref(), "DENIED");
            }
            _ => panic!("intent posture cannot manufacture collection content"),
        }
        if let Some(owners) = session.appearance_owner_snapshot.as_ref() {
            assert_eq!(
                owners.turn(),
                receipt.plan().basis().classification().turn()
            );
            assert_ne!(Some(owners.turn()), predecessor_turn);
        }
        drop(receipt);
        let _ = session.shutdown();
    }
}

fn publish(
    session: &mut crate::facade::WorthUiActiveApplicationSession,
    host: &crate::certification_support::ScriptedPresentationHost,
    frame: crate::mounting::UiPreparedMountedFrame,
    admits_text: bool,
    now: u64,
) {
    for _ in frame.surfaces() {
        if admits_text {
            host.push_native_display_presented();
        } else {
            host.push_native_display_settled_without_effects();
        }
    }
    match session.present_prepared_mounted_frame_internal(
        frame,
        worth_ui_host_contract::UiPresentationDeadline::at_tick(100),
        now,
    ) {
        crate::mounting::UiMountedFrameOutcome::Published(_)
        | crate::mounting::UiMountedFrameOutcome::Unchanged(_) => {}
        crate::mounting::UiMountedFrameOutcome::RejectedBeforeEffects(rejected) => {
            panic!(
                "text={admits_text} publication={now}: {:?}",
                rejected.rejections()
            );
        }
        crate::mounting::UiMountedFrameOutcome::AdmissionDenied(denial) => {
            panic!("text={admits_text} publication={now}: {denial:?}");
        }
        crate::mounting::UiMountedFrameOutcome::CompletionDenied(denial) => {
            panic!("text={admits_text} publication={now}: {denial:?}");
        }
        other => panic!(
            "text={admits_text} publication={now}: {:?}",
            std::mem::discriminant(&other)
        ),
    }
}

#[test]
fn observed_frame_rejects_foreign_frame_and_stale_owner_views_before_host_effects() {
    let role = fixture::role();
    let (mut session, host) = fixture::session_with_component(
        &role,
        fixture::source_with_role(Some(&role), 1),
        fixture::component(),
    );
    let (surface, _) = super::super::mounting_fixture::mount(&mut session, 1_000);
    super::close(&mut session, &role, 1, "observed-owner-bond-initial");
    session.advance_mounted_identity_frame().unwrap();
    let initial = prepare(&mut session);
    publish(&mut session, &host, initial, false, 1);
    let route = super::activation_route(&mut session, surface, 1);
    let posture = session
        .intent_postures
        .prepare(
            route.graph_node(),
            route.target(),
            crate::fact_contract::UiIntentPostureReference::Route(route.definition_id()),
            crate::fact_contract::UiIntentPostureKind::Denied,
        )
        .unwrap();
    let observation = crate::facade::entry::intent_consequence_observation::prepare_intent_consequence_observation(
        &mut session,
        crate::runtime::observation::UiIntentConsequenceObservationBatch::new(Some(posture), None, None),
    ).unwrap_or_else(|_| panic!("owner close"));
    let (frame, evidence) = session
        .prepare_observed_intent_consequence_frame(
            crate::mounting::UiMountedSemanticContentInput::empty(),
            0,
            Vec::new(),
            observation.progress,
        )
        .unwrap();
    drop(observation.set);
    // Actual minted frame identity, rather than equivalent semantic content,
    // distinguishes these two independently prepared frames.
    let other = session
        .prepare_content_rebind_frame(
            crate::mounting::UiMountedSemanticContentInput::empty(),
            session.mounted_frame_request(),
        )
        .unwrap();
    let calls = host.presentation_calls();
    assert!(matches!(
        session.present_prepared_observed_frame(
            other,
            &evidence,
            None,
            worth_ui_host_contract::UiPresentationDeadline::at_tick(100),
            2,
        ),
        Err(crate::runtime::rebind::UiRebindPreparationDenial::ConsequenceFrameMismatch)
    ));
    assert_eq!(host.presentation_calls(), calls);
    evidence.validate(&session, &frame).unwrap();

    let newer = session.begin_observation_turn().unwrap().seal().unwrap();
    session.classify_observations(newer).unwrap();
    let current_turn = session.appearance_owner_snapshot.as_ref().unwrap().turn();
    assert!(matches!(
        session.present_prepared_observed_frame(
            frame,
            &evidence,
            None,
            worth_ui_host_contract::UiPresentationDeadline::at_tick(100),
            3,
        ),
        Err(crate::runtime::rebind::UiRebindPreparationDenial::StaleConsequenceOwnerSnapshot)
    ));
    assert_eq!(host.presentation_calls(), calls);
    assert_eq!(
        session.appearance_owner_snapshot.as_ref().unwrap().turn(),
        current_turn
    );
    let _ = session.shutdown();
}
