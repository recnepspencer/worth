use crate::runtime::intent::{UiIntentInoperableCause, UiIntentStandingOperabilityUnavailable};
use worth_ui_host_contract::*;

#[test]
fn standing_observation_precedes_activation_and_tracks_stationary_dependencies_without_a_role() {
    let role = super::fixture::role();
    let source = super::fixture::source_with_role(None, 2);
    let (mut session, host) = super::fixture::session_with_source(&role, source);
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
                        != "worth_ui.runtime.bootstrap.product_root"
                })
        })
        .unwrap();
    assert!(session
        .graph()
        .snapshot()
        .nodes()
        .iter()
        .all(|node| node.appearance_role_attachment().is_none()));
    let (surface, _) = super::super::mounting_fixture::mount_graph_node(&mut session, 1_000, graph);
    session.advance_mounted_identity_frame().unwrap();
    let frame = super::prepare(&mut session);
    super::publish(&mut session, &host, frame, 1);
    let target = target_at_center(&session, surface);
    let baseline = session.intent_admission_metrics();
    let ready = session.observe_activation_operability(target).unwrap();
    assert_eq!(ready.generation(), &session.active_generation_identity());
    assert_eq!(ready.graph_node(), graph);
    assert_eq!(ready.target().node_receipt(), target.node_receipt());
    assert_eq!(
        ready.route(),
        super::fixture::ROUTE,
        "Activate must select its declared route even when Submit is also declared"
    );
    assert_eq!(ready.product_decision().unwrap().primary_cause(), None);

    session
        .update_intent_boolean_fact(&super::fixture::fact(super::fixture::MUTABLE), false)
        .unwrap();
    let readonly = session.observe_activation_operability(target).unwrap();
    assert_eq!(
        readonly.product_decision().unwrap().primary_cause(),
        Some(UiIntentInoperableCause::Readonly)
    );
    session
        .update_intent_boolean_fact(&super::fixture::fact(super::fixture::POLICY), false)
        .unwrap();
    let denied = session.observe_activation_operability(target).unwrap();
    assert_eq!(
        denied.product_decision().unwrap().primary_cause(),
        Some(UiIntentInoperableCause::PolicyDenied)
    );
    assert_eq!(
        ready.product_decision().unwrap().primary_cause(),
        None,
        "previous observation remains historical rather than changing with mutable owners"
    );
    assert_eq!(session.intent_admission_metrics(), baseline);
    assert_eq!(session.active_intent_occupancy_count_for_certification(), 0);
    assert!(
        session
            .intent_admission
            .operability_standing_snapshot()
            .is_none(),
        "read-only observation neither requires appearance demand nor mutates standing ownership"
    );

    advance_motion_epoch(&mut session, &host, target);
    assert!(
        matches!(
            session.observe_activation_operability(target),
            Err(UiIntentStandingOperabilityUnavailable::Presentation(
                crate::mounting::UiPresentedFrameBasisDenial::PresentationEpochMismatch { .. }
            ))
        ),
        "a retained node receipt cannot freshen an old physical target"
    );
    let current_target = target_at_center(&session, surface);
    assert_eq!(current_target.node_receipt(), target.node_receipt());
    assert_ne!(
        current_target.presentation().epoch(),
        target.presentation().epoch()
    );
    assert_eq!(
        session
            .observe_activation_operability(current_target)
            .unwrap()
            .product_decision()
            .unwrap()
            .primary_cause(),
        Some(UiIntentInoperableCause::PolicyDenied)
    );
    session
        .unmount_instance(current_target.mounted_instance())
        .unwrap();
    assert!(matches!(
        session.observe_activation_operability(current_target),
        Err(UiIntentStandingOperabilityUnavailable::Target(_))
    ));
    let _ = session.shutdown();
}

#[test]
fn standing_observation_rejects_missing_activation_and_matches_real_payload_evaluation() {
    let role = super::fixture::role();
    for routes in [0, 1] {
        let (mut session, host) = super::fixture::session(&role, routes);
        let (surface, _) = super::super::mounting_fixture::mount(&mut session, 1_000);
        super::close(&mut session, &role, routes, "standing-observation-initial");
        session.advance_mounted_identity_frame().unwrap();
        let frame = super::prepare(&mut session);
        super::publish(&mut session, &host, frame, 1);
        let target = target_at_center(&session, surface);
        if routes == 0 {
            assert!(matches!(
                session.observe_activation_operability(target),
                Err(UiIntentStandingOperabilityUnavailable::MissingActivationRoute)
            ));
        } else {
            let standing = session.observe_activation_operability(target).unwrap();
            let (_, actual) = super::activate(&mut session, surface, 1);
            assert_eq!(
                standing.product_decision().unwrap(),
                &actual,
                "read-only and activation paths must retain the same complete owner decision"
            );
        }
        let _ = session.shutdown();
    }
}

fn advance_motion_epoch(
    session: &mut crate::facade::WorthUiActiveApplicationSession,
    host: &crate::certification_support::ScriptedPresentationHost,
    target: crate::runtime::interaction::UiPresentedInteractionTargetView,
) {
    use crate::runtime::motion::{
        UiMotionCommitReceipt, UiMotionDeclaration, UiMotionTargetIdentity,
    };
    let presentation = target.presentation();
    let hit = session
        .mounted
        .interaction_hit_test_basis(presentation)
        .unwrap();
    let bounds = hit.rows()[0].bounds();
    let geometry = [bounds.x(), bounds.y(), bounds.width(), bounds.height()];
    // The semantic Motion track is a fixture. Actual mounted completion and
    // host settlement advance the epoch; this does not claim native raster proof.
    session
        .mounted
        .install_motion_commit(UiMotionCommitReceipt::for_sampling_test_transition(
            907,
            UiMotionTargetIdentity::from_family_owner(
                target.surface(),
                target.mounted_instance(),
                907,
            ),
            presentation,
            Some(geometry),
            true,
            Some(geometry),
            true,
            UiMotionDeclaration::portal_entrance(),
            None,
        ))
        .unwrap();
    host.push_presentation(UiHostSurfacePresentationOutcome::Presented(
        UiMountedSurfacePresentationCompletion::new(
            UiHostSurfacePresentationMode::NativeDisplay,
            UiHostPresentationEpoch::issued_by_host(2),
            UiMountedCompletedEffects::new(vec![UiMountedEffectFamily::NativePaint]),
            UiHostPresentationCostReport::default(),
        ),
    ));
    let tick = session.prepare_motion_tick(1, presentation).unwrap();
    session.present_prepared_motion_tick(tick, presentation);
    assert_eq!(
        session
            .mounted
            .current_presentation_for_surface(target.surface())
            .unwrap()
            .epoch(),
        UiHostPresentationEpoch::issued_by_host(2)
    );
}

fn target_at_center(
    session: &crate::facade::WorthUiActiveApplicationSession,
    surface: UiSemanticSurfaceIdentity,
) -> crate::runtime::interaction::UiPresentedInteractionTargetView {
    let presentation = session
        .mounted
        .current_presentation_for_surface(surface)
        .unwrap();
    let hit = session
        .mounted
        .interaction_hit_test_basis(presentation)
        .unwrap();
    assert_eq!(hit.rows().len(), 1);
    let bounds = hit.rows()[0].bounds();
    let position = UiHostSurfacePosition::viewport_logical(
        ((bounds.x() + bounds.width() / 2.0) * 1_000.0) as i64,
        ((bounds.y() + bounds.height() / 2.0) * 1_000.0) as i64,
    );
    crate::runtime::interaction::targeting::resolve_presented_target(
        &session.mounted,
        presentation,
        position,
    )
    .unwrap()
    .view()
}
