use super::*;
use crate::runtime::motion::{UiMotionCommitReceipt, UiMotionDeclaration, UiMotionTargetIdentity};

#[test]
fn stationary_pointer_refreshes_through_motion_host_settlement_and_next_appearance_close() {
    let role = super::super::fixture::role();
    let (mut session, host) = super::super::fixture::session(&role);
    let (surface, graph) = super::super::mounting_fixture::mount(&mut session, 1_000);
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
    session.mount_instance(node, neighbor).unwrap();
    crate::facade::entry::mounted_occurrence_geometry_test_support::refresh_nonoverlapping_surface_geometry(
        &mut session,
        neighbor,
    );
    super::super::close_source_turn(&mut session, &role, "automatic-pointer-initial");
    publish(&mut session, &host, 1);
    let first = presentation(&session, surface);
    let row = session
        .mounted
        .interaction_hit_test_basis(first)
        .unwrap()
        .rows()[0];
    let bounds = row.bounds();
    let inside = UiHostSurfacePosition::viewport_logical(
        ((bounds.x() + bounds.width() / 2.0) * 1000.0) as i64,
        ((bounds.y() + bounds.height() / 2.0) * 1000.0) as i64,
    );
    let outside = UiHostSurfacePosition::viewport_logical(-10000, -10000);
    admit(
        &mut session,
        first,
        1,
        UiHostPointerIdentity::new(2),
        outside,
        false,
    );
    admit(
        &mut session,
        first,
        2,
        UiHostPointerIdentity::new(1),
        inside,
        true,
    );
    let other = presentation(&session, neighbor);
    admit(
        &mut session,
        other,
        3,
        UiHostPointerIdentity::new(3),
        inside,
        false,
    );
    super::super::close_source_turn(&mut session, &role, "automatic-pointer-baseline");
    publish(&mut session, &host, 2);
    let current = presentation(&session, surface);
    let baseline = presence(&session);
    assert_target_basis(&baseline, 1, current);
    let old_report = session.interaction.presentation_refresh_snapshot();
    let target = UiMotionTargetIdentity::from_family_owner(surface, row.mounted_instance(), 601);
    // Marked semantic sampling fixture; the mounted work, host settlement,
    // interaction owner refresh, and appearance observation close are real.
    session
        .mounted
        .install_motion_commit(UiMotionCommitReceipt::for_sampling_test_transition(
            601,
            target,
            current,
            Some([
                bounds.x(),
                bounds.y() - bounds.height() * 2.0,
                bounds.width(),
                bounds.height(),
            ]),
            true,
            Some([bounds.x(), bounds.y(), bounds.width(), bounds.height()]),
            true,
            UiMotionDeclaration::portal_entrance(),
            None,
        ))
        .unwrap();
    host.push_rejected();
    let prepared = session.prepare_motion_tick(1, current).unwrap();
    session.present_prepared_motion_tick(prepared, current);
    assert_eq!(presence(&session), baseline);
    assert_eq!(
        session.interaction.presentation_refresh_snapshot(),
        old_report
    );
    host.push_native_display_presented();
    let prepared = session.prepare_motion_tick(1, current).unwrap();
    session.present_prepared_motion_tick(prepared, current);
    let after = presence(&session);
    let report = session.interaction.presentation_refresh_snapshot().unwrap();
    assert_eq!(report.hover.unwrap().retested, 1);
    assert_eq!(report.hover.unwrap().changed, 1);
    assert_eq!(report.pressed.unwrap().retested, 1);
    assert_eq!(report.pressed.unwrap().changed, 1);
    assert_eq!(
        after.primary_pointer(surface),
        Some(UiHostPointerIdentity::new(1))
    );
    for pointer in [UiHostPointerIdentity::new(2), UiHostPointerIdentity::new(3)] {
        let before = baseline
            .postures()
            .iter()
            .find(|p| p.pointer() == pointer)
            .unwrap();
        let current = after
            .postures()
            .iter()
            .find(|p| p.pointer() == pointer)
            .unwrap();
        assert!(current.appearance_dependency_eq(*before));
        assert_eq!(current.presentation(), before.presentation());
        assert_eq!(current.node_receipt(), before.node_receipt());
        assert_eq!(
            current.observation_sequence(),
            before.observation_sequence()
        );
    }
    let primary = after
        .postures()
        .iter()
        .find(|p| p.pointer() == UiHostPointerIdentity::new(1))
        .unwrap();
    assert_eq!(primary.target(), None);
    assert_eq!(
        primary.observation_sequence(),
        UiHostObservationSequence::new(2)
    );
    assert_eq!(
        session.interaction.pressed_appearance_snapshot().postures()[0].class(),
        crate::runtime::interaction::gesture::UiPressedAppearanceClass::CapturedOutside
    );
    super::super::close_source_turn(&mut session, &role, "automatic-pointer-close");
    assert_eq!(
        session
            .presentation
            .appearance_invalidation_batch()
            .unwrap()
            .selected_count(),
        1,
        "only the stationary pointer's affected consumer is selected"
    );
    // Adversarial delivery proof: an expired committed token must fail without
    // changing owners. A later physical sample with identical geometry must
    // recover by retesting, rather than blessing stale outside posture.
    let outside_snapshot = presence(&session);
    session
        .mounted
        .install_motion_commit(UiMotionCommitReceipt::for_sampling_test_transition(
            602,
            target,
            current,
            Some([bounds.x(), bounds.y(), bounds.width(), bounds.height()]),
            true,
            Some([bounds.x(), bounds.y(), bounds.width(), bounds.height()]),
            true,
            UiMotionDeclaration::portal_entrance(),
            None,
        ))
        .unwrap();
    let stale = commit_without_delivery(&mut session, &host, 2, current, 2);
    let epoch_two = stale
        .presentations()
        .into_iter()
        .find(|(candidate, _)| *candidate == surface)
        .unwrap()
        .1;
    let successor = commit_without_delivery(&mut session, &host, 3, epoch_two, 3);
    assert_eq!(successor.changes().changed_count(), 0);
    session
        .interaction
        .observe_presented_hit_transition(&stale, &session.mounted);
    assert_eq!(presence(&session), outside_snapshot);
    let denied = session.interaction.presentation_refresh_snapshot().unwrap();
    assert!(denied.hover.is_err() && denied.pressed.is_err());
    assert!(denied.hover_retry && denied.pressed_retry);
    session
        .interaction
        .observe_presented_hit_transition(&successor, &session.mounted);
    let recovered = session.interaction.presentation_refresh_snapshot().unwrap();
    assert_eq!(recovered.hover.unwrap().changed, 1);
    assert_eq!(recovered.pressed.unwrap().changed, 1);
    assert!(recovered.hover.unwrap().retested > 0);
    assert!(!recovered.hover_retry && !recovered.pressed_retry);
    assert_eq!(
        presence(&session)
            .postures()
            .iter()
            .find(|p| p.pointer() == UiHostPointerIdentity::new(1))
            .unwrap()
            .target(),
        Some(row.mounted_instance())
    );
    let epoch_three = successor
        .presentations()
        .into_iter()
        .find(|(candidate, _)| *candidate == surface)
        .unwrap()
        .1;
    assert_target_basis(&presence(&session), 1, epoch_three);
    assert_eq!(
        session
            .mounted
            .current_semantic_surface_for_presentation(epoch_three),
        Ok(surface)
    );
    for expired in [current, epoch_two] {
        assert!(session
            .mounted
            .current_semantic_surface_for_presentation(expired)
            .is_err());
    }
    super::super::close_source_turn(&mut session, &role, "automatic-pointer-recovered-close");
    publish(&mut session, &host, 3);
    assert_target_basis(&presence(&session), 1, presentation(&session, surface));
    session
        .mounted
        .current_unpublished_appearance()
        .unwrap()
        .unwrap();
    let _ = session.shutdown();
}

fn assert_target_basis(
    snapshot: &crate::runtime::interaction::UiPointerPresenceAppearanceOwnerSnapshot,
    pointer: u64,
    presentation: UiHostObservationPresentationBasis,
) {
    let posture = snapshot
        .postures()
        .iter()
        .find(|posture| posture.pointer() == UiHostPointerIdentity::new(pointer))
        .unwrap();
    let target = posture.presented_target().unwrap();
    assert_eq!(target.presentation(), presentation);
    assert_eq!(target.geometry().presentation(), presentation);
    assert_eq!(Some(target.node_receipt()), posture.node_receipt());
}

fn commit_without_delivery(
    session: &mut crate::facade::WorthUiActiveApplicationSession,
    host: &crate::certification_support::ScriptedPresentationHost,
    tick: u64,
    basis: UiHostObservationPresentationBasis,
    epoch: u64,
) -> crate::mounting::UiCommittedPresentedHitTransition {
    use worth_ui_host_contract::*;
    host.push_presentation(UiHostSurfacePresentationOutcome::Presented(
        UiMountedSurfacePresentationCompletion::new(
            UiHostSurfacePresentationMode::NativeDisplay,
            UiHostPresentationEpoch::issued_by_host(epoch),
            UiMountedCompletedEffects::new(vec![UiMountedEffectFamily::NativePaint]),
            UiHostPresentationCostReport::default(),
        ),
    ));
    let prepared = session.mounted.prepare_motion_tick(tick, basis).unwrap();
    let crate::mounting::UiMountedMotionSampleSettlement::Committed(mut receipt) = session
        .mounted
        .present_prepared_motion_tick(&session.host_session, prepared, basis)
    else {
        panic!("scripted accepted sample must commit");
    };
    receipt
        .take_hit_transition()
        .expect("physical commit issues transition")
}
