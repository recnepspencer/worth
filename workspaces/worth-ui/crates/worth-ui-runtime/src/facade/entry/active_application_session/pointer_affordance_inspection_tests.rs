use worth_ui_host_contract::*;
use worth_ui_inspection::{
    UiPointerAffordanceInspectionDecision as Decision,
    UiPointerAffordanceInspectionExpiry as Expiry, UiPointerAffordanceInspectionFamily as Family,
    UiPointerAffordanceInspectionInoperableCause as Cause,
    UiPointerAffordanceInspectionOutcome as Outcome,
    UiPointerAffordanceInspectionPresentation as Presentation,
};

#[test]
fn pointer_inspection_explains_sealed_meaning_and_rejected_publication_without_reobserving() {
    let (mut session, host, surfaces) = super::mounted_world();
    let surface = surfaces[0];
    assert_eq!(
        session.why_pointer_affordance(
            session.appearance_inspection_world(surface),
            current_target(&session, surface)
        ),
        Outcome::Unavailable
    );
    super::motion(
        &mut session,
        surface,
        1,
        1,
        UiHostPointerDeviceKind::Mouse,
        false,
    );
    let target = target(&session);
    let metrics = session.intent_admission_metrics();
    let publication = session.current_mounted_publication().unwrap().frame();
    let owner = super::snapshot(&session);
    let ready =
        found(session.why_pointer_affordance(session.appearance_inspection_world(surface), target));
    assert_eq!(ready.family, Family::Activation);
    assert_eq!(ready.presentation, Presentation::Pending);
    assert_eq!(ready.world, session.appearance_inspection_world(surface));
    assert_eq!(ready.pointer_identity, 1);
    assert_eq!(
        ready.mounted_instance_identity,
        target.mounted_instance().diagnostic_value()
    );
    assert_eq!(
        ready.node_receipt_identity,
        target.node_receipt().diagnostic_value()
    );
    assert_eq!(ready.observation_turn, owner.observation_turn());
    assert_eq!(ready.source_basis, session.capabilities().digest().as_u64());
    assert_eq!(
        ready.graph_node_digest,
        Some(
            session
                .mounted
                .current_mounted_identity_basis(target.mounted_instance())
                .unwrap()
                .graph_node_identity()
                .digest()
        )
    );
    assert_eq!(ready.route.as_deref(), Some(super::super::fixture::ROUTE));
    assert!(
        matches!(&ready.decision, Decision::Product { causes, selected_dependencies_visited, .. }
        if causes.is_empty() && *selected_dependencies_visited > 0)
    );
    assert_eq!(ready.pointer_rows_examined, 1);
    assert_eq!(
        session.why_pointer_affordance(session.appearance_inspection_world(surface), target),
        Outcome::Found(ready.clone())
    );
    assert!(owner.same_owner_snapshot(&super::snapshot(&session)));
    assert_eq!(
        session.current_mounted_publication().unwrap().frame(),
        publication
    );
    assert_eq!(session.intent_admission_metrics(), metrics);

    publish(&mut session, &host, 2, true);
    assert_eq!(
        session.why_pointer_affordance(session.appearance_inspection_world(surface), target),
        Outcome::Expired(Expiry::PresentationChanged)
    );
    let settled = found(session.why_pointer_affordance(
        session.appearance_inspection_world(surface),
        current_target(&session, surface),
    ));
    assert_eq!(settled.family, Family::Activation);
    assert_eq!(settled.presentation, Presentation::Current);
    assert_eq!(
        settled.observation_turn, ready.observation_turn,
        "accepted observation reuse needs no later owner close"
    );
    super::motion(
        &mut session,
        surface,
        2,
        1,
        UiHostPointerDeviceKind::Mouse,
        false,
    );
    let current = self::target(&session);
    assert_eq!(
        found(
            session.why_pointer_affordance(session.appearance_inspection_world(surface), current)
        )
        .presentation,
        Presentation::Current
    );

    session
        .update_intent_boolean_fact(
            &super::super::fixture::fact(super::super::fixture::MUTABLE),
            false,
        )
        .unwrap();
    // An inspection read explains the sealed owner result; it cannot rerun
    // Intent against newer mutable facts and invent a successor cursor.
    assert_eq!(
        found(
            session.why_pointer_affordance(session.appearance_inspection_world(surface), current)
        )
        .family,
        Family::Activation
    );
    super::close_source(&mut session, "pointer-inspection-readonly");
    let readonly = found(
        session.why_pointer_affordance(session.appearance_inspection_world(surface), current),
    );
    assert_eq!(readonly.family, Family::Default);
    assert_eq!(readonly.presentation, Presentation::Pending);
    assert!(
        matches!(&readonly.decision, Decision::Product { causes, .. }
        if causes.as_ref() == [Cause::Readonly])
    );
    publish(&mut session, &host, 3, false);
    assert_eq!(
        session.why_pointer_affordance(session.appearance_inspection_world(surface), current),
        Outcome::Found(readonly.clone())
    );
    assert_eq!(
        session
            .mounted
            .current_pointer_affordance_for_test(surface)
            .unwrap()
            .family(),
        UiPointerAffordanceFamily::Activation
    );
    publish(&mut session, &host, 4, true);
    assert_eq!(
        session
            .mounted
            .current_pointer_affordance_for_test(surface)
            .unwrap()
            .family(),
        UiPointerAffordanceFamily::Default
    );
    super::motion(
        &mut session,
        surface,
        3,
        1,
        UiHostPointerDeviceKind::Mouse,
        false,
    );
    let accepted = found(session.why_pointer_affordance(
        session.appearance_inspection_world(surface),
        self::target(&session),
    ));
    assert_eq!(accepted.family, Family::Default);
    assert_eq!(accepted.presentation, Presentation::Current);
    assert_eq!(
        ready.family,
        Family::Activation,
        "owned historical explanation stays inert"
    );

    session
        .update_intent_boolean_fact(
            &super::super::fixture::fact(super::super::fixture::POLICY),
            false,
        )
        .unwrap();
    super::close_source(&mut session, "pointer-inspection-policy");
    let policy = found(session.why_pointer_affordance(
        session.appearance_inspection_world(surface),
        self::target(&session),
    ));
    assert!(matches!(policy.decision, Decision::Product { causes, .. }
        if causes.as_ref() == [Cause::PolicyDenied, Cause::Readonly]));
    let (other, _, _) = super::mounted_world();
    assert_eq!(
        other.why_pointer_affordance(ready.world, self::target(&session)),
        Outcome::WrongWorld
    );
    let _ = other.shutdown();
    let current = current_target(&session, surface);
    let world = session.appearance_inspection_world(surface);
    session.deregister_host_surface(current.binding()).unwrap();
    assert_eq!(
        session.why_pointer_affordance(world, current),
        Outcome::Expired(Expiry::BindingChanged)
    );
    let _ = session.shutdown();
    assert_eq!(host.pending_presentation_count(), 0);
}

fn target(
    session: &crate::facade::WorthUiActiveApplicationSession,
) -> crate::runtime::interaction::UiPresentedInteractionTargetView {
    super::snapshot(session)
        .active_projections()
        .next()
        .unwrap()
        .presented_target()
        .unwrap()
}

#[test]
fn invalidated_pointer_snapshot_is_diagnostic_only() {
    // Owner/query branch proof. The lifecycle integration uses real surface
    // deregistration above; this test does not claim to exercise host rebind.
    let (mut session, host, surfaces) = super::mounted_world();
    let surface = surfaces[0];
    super::motion(
        &mut session,
        surface,
        1,
        1,
        UiHostPointerDeviceKind::Mouse,
        false,
    );
    let target = target(&session);
    let world = session.appearance_inspection_world(surface);
    session
        .pointer_affordance_snapshot
        .as_mut()
        .unwrap()
        .invalidate_surface(surface);
    let before = super::snapshot(&session);
    assert_eq!(
        session.why_pointer_affordance(world, target),
        Outcome::Expired(Expiry::SurfaceInvalidated)
    );
    let after = super::snapshot(&session);
    assert!(before.same_owner_snapshot(&after));
    assert_eq!(after.active_projections().count(), 0);
    let _ = session.shutdown();
    assert_eq!(host.pending_presentation_count(), 0);
}

fn found(outcome: Outcome) -> worth_ui_inspection::UiPointerAffordanceInspectionExplanation {
    match outcome {
        Outcome::Found(explanation) => explanation,
        other => panic!("current sealed pointer explanation: {other:?}"),
    }
}

pub(super) fn current_target(
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
    let [row] = hit.rows() else {
        panic!("one mounted target")
    };
    let bounds = row.bounds();
    let position = UiHostSurfacePosition::viewport_logical(
        ((bounds.x() + bounds.width() / 2.0) * 1_000.0) as i64,
        ((bounds.y() + bounds.height() / 2.0) * 1_000.0) as i64,
    );
    crate::runtime::interaction::targeting::resolve_presented_target(
        &session.mounted,
        presentation,
        position,
        &mut Default::default(),
    )
    .unwrap()
    .view()
}

fn publish(
    session: &mut crate::facade::WorthUiActiveApplicationSession,
    host: &crate::certification_support::ScriptedPresentationHost,
    now: u64,
    accept: bool,
) {
    for _ in 0..3 {
        if accept {
            host.push_presentation(UiHostSurfacePresentationOutcome::Presented(
                UiMountedSurfacePresentationCompletion::new(
                    UiHostSurfacePresentationMode::NativeDisplay,
                    UiHostPresentationEpoch::issued_by_host(now),
                    // Cursor-only and unchanged surfaces have no raster paint.
                    UiMountedCompletedEffects::new(Vec::new()),
                    UiHostPresentationCostReport::default(),
                ),
            ));
        } else {
            host.push_rejected();
        }
    }
    let request = session.mounted_frame_request();
    let outcome = session
        .execute_mounted_frame(
            request,
            UiPresentationDeadline::at_tick(u64::MAX),
            now,
            |_| {},
        )
        .unwrap_or_else(|_| panic!("ordinary pointer publication prepares"));
    use crate::mounting::UiMountedFrameOutcome as Frame;
    match outcome {
        Frame::Published(_) => assert!(accept),
        Frame::RejectedBeforeEffects(denial) => {
            assert!(!accept, "host rejected {now}: {:?}", denial.rejections())
        }
        Frame::AdmissionDenied(denial) => panic!("admission at {now}: {denial:?}"),
        Frame::CompletionDenied(denial) => panic!("completion at {now}: {denial:?}"),
        Frame::Unchanged(_) => panic!("expected changed pointer at {now}"),
        Frame::PresentationIndeterminate(denial) => {
            panic!("indeterminate at {now}: {:?}", denial.report())
        }
        Frame::InFlight(_) => panic!("in flight at {now}"),
        Frame::RetentionDenied(denial) => panic!("retention at {now}: {denial:?}"),
        Frame::Superseded(_) => panic!("superseded at {now}"),
        Frame::Reconciled(_) => panic!("reconciled at {now}"),
    }
}
