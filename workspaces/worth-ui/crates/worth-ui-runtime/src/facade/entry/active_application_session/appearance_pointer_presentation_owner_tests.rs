#[path = "appearance_pointer_presentation_owner_tests/automatic.rs"]
mod automatic;
#[path = "appearance_pointer_presentation_owner_tests/motion_index.rs"]
mod motion_index;

use crate::runtime::interaction::{
    UiPointerPresenceGeometry, UiPointerPresenceGeometryCandidate,
    UiPointerPresencePresentationTrigger,
};
use worth_ui_host_contract::*;

#[test]
fn pointer_presentation_refresh_preserves_admitted_surface_and_primary_authority() {
    let role = super::fixture::role();
    let (mut session, host) = super::fixture::session(&role);
    let (surface, graph) = super::mounting_fixture::mount(&mut session, 1_000);
    let neighbor = session.create_semantic_surface().unwrap();
    session
        .register_host_surface(
            neighbor,
            crate::facade::mounted::UiHostSurfacePresentationMode::NativeDisplay,
            crate::facade::mounted::UiSurfaceBindingProfile::new(
                1_000,
                crate::facade::mounted::UiSurfaceBindingCoordinatePosture::LogicalPoints,
                1,
            )
            .unwrap(),
        )
        .unwrap();
    let node = session.mounted_graph_node(graph).unwrap();
    let neighbor_target = session.mount_instance(node, neighbor).unwrap();
    crate::facade::entry::mounted_occurrence_geometry_test_support::refresh_nonoverlapping_surface_geometry(
        &mut session,
        neighbor,
    );
    super::close_source_turn(&mut session, &role, "pointer-refresh-initial");
    publish(&mut session, &host, 1);
    let first = presentation(&session, surface);
    let other = presentation(&session, neighbor);
    let basis = session.mounted.interaction_hit_test_basis(first).unwrap();
    let row = *basis
        .rows()
        .iter()
        .find(|row| {
            session
                .mounted
                .current_mounted_identity_basis(row.mounted_instance())
                .unwrap()
                .graph_node_identity()
                == graph
        })
        .unwrap();
    let inside = UiHostSurfacePosition::viewport_logical(
        ((row.bounds().x() + row.bounds().width() * 0.5) * 1_000.0) as i64,
        ((row.bounds().y() + row.bounds().height() * 0.5) * 1_000.0) as i64,
    );
    let outside = UiHostSurfacePosition::viewport_logical(-1_000, -1_000);
    let traveling = UiHostPointerIdentity::new(9);

    let entered_empty = admit(&mut session, first, 1, traveling, outside, false);
    let transition = &entered_empty.pointer_presence_transitions()[0];
    assert_eq!(transition.current_surface(), Some(surface));
    assert_eq!(transition.current(), None);
    assert_eq!(presence(&session).primary_pointer(surface), Some(traveling));
    let crossed_empty = admit(&mut session, other, 2, traveling, outside, false);
    let transition = &crossed_empty.pointer_presence_transitions()[0];
    assert_eq!(transition.previous_surface(), Some(surface));
    assert_eq!(transition.current_surface(), Some(neighbor));
    assert_eq!(transition.current(), None);
    assert_eq!(presence(&session).primary_pointer(surface), None);
    assert_eq!(
        presence(&session).primary_pointer(neighbor),
        Some(traveling)
    );

    let before_denial = presence(&session);
    let invalid = UiHostSurfacePosition::new(
        UiHostSurfacePositionBasis::new(
            UiHostSurfaceCoordinateSpace::Viewport,
            UiHostSurfaceCoordinateUnit::PhysicalPixel,
        ),
        inside.x_subpixels(),
        inside.y_subpixels(),
    );
    let denied = ingest(&mut session, other, 3, traveling, invalid, false);
    assert_eq!(denied.pointer_presence_denials(), &[
        crate::runtime::interaction::UiPointerPresenceAdmissionDenial::Targeting {
            pointer: traveling,
            denial: crate::runtime::interaction::targeting::UiInteractionTargetingDenial::UnsupportedPositionBasis(invalid.basis()),
        }
    ]);
    assert!(denied.pointer_presence_transitions().is_empty());
    assert_eq!(
        presence(&session),
        before_denial,
        "denied input cannot become an outside observation"
    );

    admit(&mut session, other, 4, traveling, inside, true);
    // Reverse numeric and admission order so iteration cannot masquerade as primary selection.
    let older = UiHostPointerIdentity::new(2);
    let primary = UiHostPointerIdentity::new(1);
    admit(&mut session, first, 5, older, inside, true);
    admit(&mut session, first, 6, primary, inside, true);
    let previous = presence(&session);
    assert_eq!(previous.primary_pointer(surface), Some(primary));
    assert_eq!(previous.primary_pointer(neighbor), Some(traveling));
    let neighbor_posture = *previous
        .postures()
        .iter()
        .find(|p| p.pointer() == traveling)
        .unwrap();
    assert_eq!(neighbor_posture.target(), Some(neighbor_target));
    let pressed_before = session.interaction.pressed_appearance_snapshot();
    assert_eq!(pressed_before.postures().len(), 3);
    let neighbor_pressed = *pressed_before
        .postures()
        .iter()
        .find(|p| p.pointer() == traveling)
        .unwrap();

    super::close_source_turn(&mut session, &role, "pointer-refresh-successor");
    publish(&mut session, &host, 2);
    let successor = presentation(&session, surface);
    let automatic = session.interaction.presentation_refresh_snapshot().unwrap();
    assert_eq!(automatic.hover.unwrap().retested, 0);
    assert_eq!(automatic.pressed.unwrap().retested, 0);
    assert_eq!(automatic.hover.unwrap().evidence_refreshed, 3);
    assert_eq!(automatic.pressed.unwrap().evidence_refreshed, 3);
    assert_eq!(
        presence(&session)
            .postures()
            .iter()
            .find(|p| p.pointer() == traveling)
            .unwrap()
            .target(),
        neighbor_posture.target()
    );
    assert_eq!(
        session
            .interaction
            .pressed_appearance_snapshot()
            .postures()
            .iter()
            .find(|p| p.pointer() == traveling)
            .unwrap()
            .class(),
        neighbor_pressed.class()
    );
    let neighbor_posture = *presence(&session)
        .postures()
        .iter()
        .find(|p| p.pointer() == traveling)
        .unwrap();
    let neighbor_pressed = *session
        .interaction
        .pressed_appearance_snapshot()
        .postures()
        .iter()
        .find(|p| p.pointer() == traveling)
        .unwrap();

    assert_ne!(
        successor, first,
        "the proof must cross a real presentation succession"
    );
    let wrong_surface = UiHostObservationPresentationBasis::new(
        other.host_surface(),
        successor.frame(),
        successor.binding(),
        successor.epoch(),
    );
    for (invalid, denial) in [
        (
            first,
            crate::runtime::interaction::UiInteractionTargetingDenial::ExpiredPresentation,
        ),
        (
            wrong_surface,
            crate::runtime::interaction::UiInteractionTargetingDenial::BindingNotPresented,
        ),
    ] {
        let trigger =
            UiPointerPresencePresentationTrigger::new(invalid, &[row.mounted_instance()]).unwrap();
        let presence_before = presence(&session);
        let pressed_before = session.interaction.pressed_appearance_snapshot();
        assert_eq!(
            session
                .interaction
                .observe_committed_presentation(&trigger, &session.mounted),
            (Err(denial), Err(denial))
        );
        assert_eq!(presence(&session), presence_before);
        assert_eq!(
            session.interaction.pressed_appearance_snapshot(),
            pressed_before
        );
    }
    let successor_basis = session
        .mounted
        .interaction_hit_test_basis(successor)
        .unwrap();
    let successor_row = *successor_basis
        .rows()
        .iter()
        .find(|candidate| candidate.mounted_instance() == row.mounted_instance())
        .unwrap();
    // This invokes the owner boundary explicitly. Publication-trigger production is a separate proof.
    let trigger = UiPointerPresencePresentationTrigger::new_with_geometry(
        successor,
        &[UiPointerPresenceGeometryCandidate::new(
            row.mounted_instance(),
            Some(UiPointerPresenceGeometry::new(
                row.bounds(),
                row.clip_bounds(),
            )),
            Some(UiPointerPresenceGeometry::new(
                successor_row.bounds(),
                successor_row.clip_bounds(),
            )),
        )],
    )
    .unwrap();
    assert_eq!(
        session
            .interaction
            .observe_committed_presentation(&trigger, &session.mounted),
        (Ok(0), Ok(0)),
        "automatic publication already refreshed both owners; the explicit replay changes nothing"
    );
    let refreshed = presence(&session);
    assert_eq!(refreshed.owner_revision(), previous.owner_revision());
    assert_eq!(refreshed.primary_pointer(surface), Some(primary));
    assert_eq!(refreshed.primary_pointer(neighbor), Some(traveling));
    assert_eq!(
        *refreshed
            .postures()
            .iter()
            .find(|p| p.pointer() == traveling)
            .unwrap(),
        neighbor_posture
    );
    for pointer in [older, primary] {
        let before = previous
            .postures()
            .iter()
            .find(|p| p.pointer() == pointer)
            .unwrap();
        let after = refreshed
            .postures()
            .iter()
            .find(|p| p.pointer() == pointer)
            .unwrap();
        assert_eq!(after.target(), before.target());
        assert_eq!(after.observation_sequence(), before.observation_sequence());
        assert_eq!(after.presentation(), successor);
        assert_eq!(after.node_receipt(), Some(successor_row.node_receipt()));
    }
    let pressed_after = session.interaction.pressed_appearance_snapshot();
    let neighbor_after = *pressed_after
        .postures()
        .iter()
        .find(|p| p.pointer() == traveling)
        .unwrap();
    assert_eq!(
        neighbor_after.presentation(),
        neighbor_pressed.presentation()
    );
    assert_eq!(neighbor_after.target(), neighbor_pressed.target());
    assert_eq!(neighbor_after.class(), neighbor_pressed.class());
    for posture in pressed_after
        .postures()
        .iter()
        .filter(|p| p.pointer() != traveling)
    {
        assert_eq!(posture.presentation(), successor);
        assert_eq!(posture.target(), row.mounted_instance());
        assert_eq!(
            posture.class(),
            crate::runtime::interaction::gesture::UiPressedAppearanceClass::ArmedInside
        );
    }
    let _ = session.shutdown();
}

fn presence(
    session: &crate::facade::WorthUiActiveApplicationSession,
) -> crate::runtime::interaction::UiPointerPresenceAppearanceOwnerSnapshot {
    session
        .interaction
        .pointer_presence_appearance_snapshot()
        .unwrap()
}

fn presentation(
    session: &crate::facade::WorthUiActiveApplicationSession,
    surface: UiSemanticSurfaceIdentity,
) -> UiHostObservationPresentationBasis {
    session
        .mounted
        .current_publication()
        .unwrap()
        .presentation_for_surface(surface)
        .unwrap()
}

fn admit(
    session: &mut crate::facade::WorthUiActiveApplicationSession,
    presentation: UiHostObservationPresentationBasis,
    sequence: u64,
    pointer: UiHostPointerIdentity,
    position: UiHostSurfacePosition,
    press: bool,
) -> crate::runtime::interaction::UiInteractionBatchReceipt {
    let receipt = ingest(session, presentation, sequence, pointer, position, press);
    assert!(receipt.pointer_presence_denials().is_empty());
    assert_eq!(receipt.pointer_presence_transitions().len(), 1);
    receipt
}

fn ingest(
    session: &mut crate::facade::WorthUiActiveApplicationSession,
    presentation: UiHostObservationPresentationBasis,
    sequence: u64,
    pointer: UiHostPointerIdentity,
    position: UiHostSurfacePosition,
    press: bool,
) -> crate::runtime::interaction::UiInteractionBatchReceipt {
    let batch = super::pointer_batch(
        session.host_session.identity().as_u64(),
        presentation,
        sequence,
        pointer,
        position,
        press.then_some(UiHostPointerButtonTransition::Pressed),
        press,
    );
    let ingress = session.admit_host_interaction_batch(batch);
    let crate::facade::interaction::UiHostInteractionIngressOutcome::Applied(receipt) = ingress
    else {
        panic!("pointer owner proof must reach interaction admission: {ingress:?}");
    };
    receipt
}

fn publish(
    session: &mut crate::facade::WorthUiActiveApplicationSession,
    host: &crate::certification_support::ScriptedPresentationHost,
    now: u64,
) {
    if now == 1 {
        session.advance_mounted_identity_frame().unwrap();
    }
    let frame = session
        .prepare_mounted_frame_with_application_presentation(
            crate::mounting::UiMountedFrameRequest::all_bound_surfaces(),
            |_| {},
        )
        .unwrap_or_else(|_| panic!("pointer proof frame prepares"));
    for _ in frame.surfaces() {
        if now == 1 {
            host.push_native_display_presented();
        } else {
            host.push_native_display_settled_without_effects();
        }
    }
    let outcome = session.present_prepared_mounted_frame_internal(
        frame,
        UiPresentationDeadline::at_tick(100),
        now,
    );
    match outcome {
        crate::mounting::UiMountedFrameOutcome::Published(_)
        | crate::mounting::UiMountedFrameOutcome::Unchanged(_) => {}
        crate::mounting::UiMountedFrameOutcome::RetentionDenied(rejection) => {
            panic!("turn {now} retention: {:?}", rejection.denial())
        }
        crate::mounting::UiMountedFrameOutcome::AdmissionDenied(rejection) => {
            panic!("turn {now} admission: {:?}", rejection.denial())
        }
        crate::mounting::UiMountedFrameOutcome::CompletionDenied(denial) => {
            panic!("turn {now} completion: {denial:?}")
        }
        crate::mounting::UiMountedFrameOutcome::RejectedBeforeEffects(rejection) => {
            panic!("turn {now} host rejection: {:?}", rejection.rejections())
        }
        crate::mounting::UiMountedFrameOutcome::InFlight(_) => panic!("turn {now} in flight"),
        crate::mounting::UiMountedFrameOutcome::PresentationIndeterminate(_) => {
            panic!("turn {now} indeterminate")
        }
        crate::mounting::UiMountedFrameOutcome::Superseded(_) => panic!("turn {now} superseded"),
        crate::mounting::UiMountedFrameOutcome::Reconciled(_) => panic!("turn {now} reconciled"),
    }
}
