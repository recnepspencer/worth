use worth_ui::facade::interaction::{
    UiDismissInteractionCause, UiInteractionTargetingDenial, UiInteractionTransition,
    UiPointerGestureContinuityKind, UiPointerGestureStopReason, UiSemanticInteraction,
};

#[test]
fn admitted_primary_press_emits_exact_outside_press_dismissal_evidence() {
    let mut world = InteractionWorld::canonical();
    let receipt = applied(world.button(41, 3, UiHostPointerButtonTransition::Pressed, [20, 20]));
    assert!(matches!(
        receipt.transitions(),
        [UiInteractionTransition::PointerPressed(_), UiInteractionTransition::DismissRequested(dismissal)]
            if dismissal.presentation() == world.presentation
                && matches!(dismissal.cause(), UiDismissInteractionCause::OutsidePress(position)
                    if [position.x_subpixels(), position.y_subpixels()]
                        == [20 * UI_HOST_SURFACE_POSITION_SUBPIXELS_PER_UNIT; 2])
    ));
    let _ = world.session.shutdown();
}
use worth_ui::facade::observation_report::{
    UiHostObservationMountedBasis, UiHostObservationPresentationBasis,
    UiHostObservationReportDenial, UiHostPointerButtonTransition,
    UI_HOST_SURFACE_POSITION_SUBPIXELS_PER_UNIT,
};
use worth_ui_test_support::{
    WorthUiMountedIdentityCertificationExt, WorthUiMountedInteractionLifecycleCertificationExt,
};

use super::super::interaction_world::InteractionWorld;
use super::assertions::{
    applied, assert_completion, assert_rank, assert_stop, assert_targeting_stop, denied,
    pointer_gesture,
};
use super::oracle::{expected_target, ExpectedTarget};

#[test]
fn presented_geometry_adjudicates_overlap_clip_and_half_open_edges() {
    let mut canonical = InteractionWorld::canonical();
    assert_rank(
        canonical.button(1, 1, UiHostPointerButtonTransition::Pressed, [20, 20]),
        expected_target([20, 20], false),
    );
    assert_completion(
        canonical.button(1, 1, UiHostPointerButtonTransition::Released, [20, 20]),
        UiPointerGestureContinuityKind::ExactPresentation,
    );
    assert_rank(
        canonical.button(2, 1, UiHostPointerButtonTransition::Pressed, [10, 20]),
        expected_target([10, 20], false),
    );
    assert_completion(
        canonical.button(2, 1, UiHostPointerButtonTransition::Released, [10, 20]),
        UiPointerGestureContinuityKind::ExactPresentation,
    );
    assert_targeting_stop(
        canonical.button(3, 1, UiHostPointerButtonTransition::Pressed, [152, 20]),
        UiInteractionTargetingDenial::NoTarget {
            hit_test_rows_considered: 0,
        },
    );
    let _ = canonical.session.shutdown();

    let mut clipped = InteractionWorld::clipped();
    let outer = clipped
        .hit_rows
        .iter()
        .find(|row| row.order().rank() == 1)
        .expect("the production clipped world retains its outer hit row");
    assert_eq!([outer.bounds().x(), outer.bounds().width()], [8.0, 144.0]);
    assert_eq!(
        [outer.clip_bounds().x(), outer.clip_bounds().width()],
        [20.0, 120.0]
    );
    assert_targeting_stop(
        clipped.button(1, 1, UiHostPointerButtonTransition::Pressed, [10, 20]),
        match expected_target([10, 20], true) {
            ExpectedTarget::None => UiInteractionTargetingDenial::NoTarget {
                hit_test_rows_considered: 0,
            },
            ExpectedTarget::Rank(_) => panic!("independent oracle must exclude the clipped point"),
        },
    );
    let _ = clipped.session.shutdown();
}

#[test]
fn continuity_is_owner_issued_across_exact_and_successor_presentations() {
    let mut world = InteractionWorld::canonical();
    assert_rank(
        world.button(1, 4, UiHostPointerButtonTransition::Pressed, [20, 20]),
        ExpectedTarget::Rank(0),
    );
    let predecessor = world.presentation;
    world.publish_successor();
    assert_ne!(world.presentation, predecessor);
    let completed = applied(world.button(1, 4, UiHostPointerButtonTransition::Released, [20, 20]));
    let gesture = match &completed.transitions()[0] {
        UiInteractionTransition::Semantic(UiSemanticInteraction::Activate(activation)) => {
            pointer_gesture(activation.source())
        }
        other => panic!("same incarnation across a successor must complete, got {other:?}"),
    };
    assert_eq!(
        gesture.continuity(),
        UiPointerGestureContinuityKind::OwnerWitnessedSuccessor
    );
    assert_eq!(
        gesture.pressed_target().mounted_instance(),
        gesture.released_target().mounted_instance()
    );

    assert_rank(
        world.button(2, 4, UiHostPointerButtonTransition::Pressed, [20, 20]),
        ExpectedTarget::Rank(0),
    );
    assert_stop(
        world.button(2, 4, UiHostPointerButtonTransition::Released, [10, 20]),
        UiPointerGestureStopReason::MountedIncarnationChanged,
    );

    assert_rank(
        world.button(3, 4, UiHostPointerButtonTransition::Pressed, [20, 20]),
        ExpectedTarget::Rank(0),
    );
    let motion = applied(world.motion(3, 4, [159, 95]));
    assert_eq!(motion.ignored_reports(), 1);
    assert_completion(
        world.button(3, 4, UiHostPointerButtonTransition::Released, [20, 20]),
        UiPointerGestureContinuityKind::ExactPresentation,
    );
    let _ = world.session.shutdown();
}

#[test]
fn stale_presentation_epoch_stops_without_current_coordinate_retargeting() {
    let mut stale_epoch = InteractionWorld::canonical();
    assert_rank(
        stale_epoch.button(1, 1, UiHostPointerButtonTransition::Pressed, [20, 20]),
        ExpectedTarget::Rank(0),
    );
    let invalid_presentation = UiHostObservationPresentationBasis::new(
        stale_epoch.presentation.host_surface(),
        stale_epoch.presentation.frame(),
        stale_epoch.binding,
        worth_ui_host_contract::UiHostPresentationEpoch::issued_by_host(
            stale_epoch.presentation.epoch().diagnostic_value() + 1,
        ),
    );
    let epoch_denial = denied(stale_epoch.button_at_presentation(
        invalid_presentation,
        1,
        UiHostPointerButtonTransition::Released,
        [20, 20],
    ));
    assert_eq!(
        epoch_denial.denial(),
        UiHostObservationReportDenial::PresentationEpochMismatch
    );
    assert_eq!(epoch_denial.settlement().stops().len(), 1);
    assert_eq!(
        epoch_denial.settlement().stops()[0].reason(),
        UiPointerGestureStopReason::InvalidObservation
    );
    assert_eq!(epoch_denial.settlement().final_state().active_gestures(), 0);
    let _ = stale_epoch.session.shutdown();
}

#[test]
fn stale_node_receipt_cannot_reanimate_a_successor_presentation() {
    let mut stale_receipt = InteractionWorld::canonical();
    let press =
        applied(stale_receipt.button(1, 1, UiHostPointerButtonTransition::Pressed, [20, 20]));
    let pressed_target = match &press.transitions()[0] {
        UiInteractionTransition::PointerPressed(press) => press.target(),
        other => panic!("expected a targeted press, got {other:?}"),
    };
    let stale_basis = UiHostObservationMountedBasis::new(
        pressed_target.mounted_instance(),
        pressed_target.node_receipt(),
    );
    assert_completion(
        stale_receipt.button(1, 1, UiHostPointerButtonTransition::Released, [20, 20]),
        UiPointerGestureContinuityKind::ExactPresentation,
    );
    stale_receipt.publish_successor();
    let receipt_denial = denied(stale_receipt.button_with_mounted_basis(
        stale_basis,
        2,
        UiHostPointerButtonTransition::Pressed,
        [20, 20],
    ));
    assert_eq!(
        receipt_denial.denial(),
        UiHostObservationReportDenial::NodeReceiptMismatch
    );
    assert!(receipt_denial.settlement().stops().is_empty());
    let _ = stale_receipt.session.shutdown();
}

#[test]
fn foreign_binding_cannot_borrow_local_presentation_identity() {
    let mut local = InteractionWorld::canonical();
    let foreign = InteractionWorld::canonical();
    let foreign_presentation = UiHostObservationPresentationBasis::new(
        local.presentation.host_surface(),
        local.presentation.frame(),
        foreign.binding,
        local.presentation.epoch(),
    );
    let foreign_denial = denied(local.button_at_presentation(
        foreign_presentation,
        1,
        UiHostPointerButtonTransition::Pressed,
        [20, 20],
    ));
    assert_eq!(
        foreign_denial.denial(),
        UiHostObservationReportDenial::BindingNotPresented
    );
    let _ = local.session.shutdown();
    let _ = foreign.session.shutdown();
}

#[test]
fn incompatible_remount_cannot_reanimate_a_retained_presented_row() {
    let mut world = InteractionWorld::canonical();
    let press = applied(world.button(1, 1, UiHostPointerButtonTransition::Pressed, [20, 20]));
    let target = match &press.transitions()[0] {
        UiInteractionTransition::PointerPressed(press) => press.target(),
        other => panic!("expected a targeted press, got {other:?}"),
    };
    let retired = target.mounted_instance();
    let surface = target.surface();
    let graph_node = world
        .session
        .inspect_mounted_identity()
        .mounted_instances()
        .iter()
        .find(|instance| instance.identity() == retired)
        .expect("the targeted incarnation is live before unmount")
        .basis()
        .graph_node_identity();
    let handle = world.session.mounted_graph_node(graph_node).unwrap();
    let settlement = world
        .session
        .unmount_instance_with_interaction_receipt(retired)
        .unwrap();
    assert_eq!(settlement.stops().len(), 1);
    assert_eq!(
        settlement.stops()[0].reason(),
        UiPointerGestureStopReason::MountedInstanceRemoved
    );
    let replacement = world.session.mount_instance(handle, surface).unwrap();
    assert_ne!(replacement, retired);

    assert_targeting_stop(
        world.button_at_presentation(
            world.presentation,
            2,
            UiHostPointerButtonTransition::Pressed,
            [20, 20],
        ),
        UiInteractionTargetingDenial::MountedInstanceNoLongerCurrent,
    );
    let _ = world.session.shutdown();
}
