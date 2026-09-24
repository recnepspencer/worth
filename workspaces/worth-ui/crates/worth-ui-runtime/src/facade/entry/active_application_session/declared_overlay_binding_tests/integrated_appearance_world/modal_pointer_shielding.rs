use super::session::World;
use crate::runtime::interaction::{UiInteractionTransition, UiSemanticInteraction};
use worth_ui_host_contract::*;

#[test]
fn accepted_sibling_modal_blocks_older_portal_content_and_queued_clicks() {
    let mut world = World::launch();
    let frame = world.prepare();
    world.publish(frame, 1, true);
    world.open(0, "overlay.menu", None, 10);
    let before = world
        .session
        .mounted
        .current_presentation_for_surface(world.surfaces[0])
        .unwrap();
    let rows = world
        .session
        .mounted
        .interaction_hit_test_basis(before.basis())
        .unwrap();
    let content = rows
        .rows()
        .iter()
        .find(|row| row.mounted_instance() == world.instances[4])
        .unwrap();
    let bounds = content.bounds();
    let position = UiHostSurfacePosition::viewport_logical(
        ((bounds.x() + bounds.width() / 2.0) * 1_000.0) as i64,
        ((bounds.y() + bounds.height() / 2.0) * 1_000.0) as i64,
    );
    assert!(click(&mut world, before.basis(), 1, position)
        .iter()
        .flat_map(|receipt| receipt.transitions())
        .any(|event| matches!(
            event,
            UiInteractionTransition::Semantic(UiSemanticInteraction::Activate(_))
        )));

    // Both are roots (equal nesting depth). The later modal blocks the older menu.
    world.open(1, "overlay.child", None, 20);
    let current = world
        .session
        .mounted
        .current_presentation_for_surface(world.surfaces[0])
        .unwrap();
    for (presentation, sequence) in [(current, 3), (before, 5)] {
        assert!(
            !click(&mut world, presentation.basis(), sequence, position)
                .iter()
                .flat_map(|receipt| receipt.transitions())
                .any(|event| matches!(
                    event,
                    UiInteractionTransition::PointerPressed(_)
                        | UiInteractionTransition::Semantic(_)
                )),
            "accepted owner order must shield the older sibling, including queued input"
        );
    }
    let _ = world.session.shutdown();
}

#[test]
fn accepted_modal_blocks_background_activation_but_keeps_content_and_dismiss_input() {
    let mut world = World::launch();
    let frame = world.prepare();
    world.publish(frame, 1, true);
    let before = world
        .session
        .mounted
        .current_presentation_for_surface(world.surfaces[0])
        .unwrap();
    let background = UiHostSurfacePosition::viewport_logical(410_000, 85_000);
    let baseline = click(&mut world, before.basis(), 1, background);
    assert!(baseline
        .iter()
        .flat_map(|receipt| receipt.transitions())
        .any(|event| matches!(
            event,
            UiInteractionTransition::Semantic(UiSemanticInteraction::Activate(_))
        )));

    world.open(0, "overlay.child", None, 10);
    let current = world
        .session
        .mounted
        .current_presentation_for_surface(world.surfaces[0])
        .unwrap();
    let blocked = click(&mut world, current.basis(), 3, background);
    assert!(!blocked
        .iter()
        .flat_map(|receipt| receipt.transitions())
        .any(|event| matches!(
            event,
            UiInteractionTransition::PointerPressed(_) | UiInteractionTransition::Semantic(_)
        )));
    assert_eq!(
        blocked
            .iter()
            .flat_map(|receipt| receipt.transitions())
            .filter(|event| matches!(event, UiInteractionTransition::DismissRequested(_)))
            .count(),
        1,
        "outside dismissal remains an input even with no admitted target"
    );

    let queued = click(&mut world, before.basis(), 5, background);
    assert!(
        !queued
            .iter()
            .flat_map(|receipt| receipt.transitions())
            .any(|event| matches!(
                event,
                UiInteractionTransition::PointerPressed(_) | UiInteractionTransition::Semantic(_)
            )),
        "queued predecessor input cannot bypass the accepted modal"
    );

    let rows = world
        .session
        .mounted
        .interaction_hit_test_basis(current.basis())
        .unwrap();
    let content = rows
        .rows()
        .iter()
        .find(|row| row.mounted_instance() == world.instances[4])
        .unwrap();
    let bounds = content.bounds();
    let inside = UiHostSurfacePosition::viewport_logical(
        ((bounds.x() + bounds.width() / 2.0) * 1_000.0) as i64,
        ((bounds.y() + bounds.height() / 2.0) * 1_000.0) as i64,
    );
    let admitted = click(&mut world, current.basis(), 7, inside);
    assert!(admitted
        .iter()
        .flat_map(|receipt| receipt.transitions())
        .any(|event| matches!(
            event,
            UiInteractionTransition::Semantic(UiSemanticInteraction::Activate(_))
        )));
    let _ = world.session.shutdown();
}

fn click(
    world: &mut World,
    presentation: UiHostObservationPresentationBasis,
    start: u64,
    position: UiHostSurfacePosition,
) -> Vec<crate::runtime::interaction::UiInteractionBatchReceipt> {
    let mut events = Vec::new();
    for (offset, transition) in [
        UiHostPointerButtonTransition::Pressed,
        UiHostPointerButtonTransition::Released,
    ]
    .into_iter()
    .enumerate()
    {
        let UiHostProtocolNegotiation::Compatible(protocol) =
            UiHostProtocolContract::current().negotiate()
        else {
            unreachable!()
        };
        let sequence = UiHostObservationSequence::new(start + offset as u64);
        let batch = UiHostObservationBatch::new(UiHostObservationBatchInput {
            protocol,
            host_session: world.session.host_session.identity().as_u64(),
            presentation,
            sequences: UiHostObservationSequenceRange::new(sequence, sequence),
            loss: UiHostObservationLoss::Complete,
            reports: vec![UiHostObservationReport::new(
                sequence,
                UiHostObservationTimeBasis::HostMonotonicMillis(sequence.value()),
                UiHostObservationPayload::PointerButton {
                    pointer: UiHostPointerIdentity::new(1),
                    capture_epoch: UiHostPointerCaptureEpoch::new(1),
                    button: UiHostPointerButton::Primary,
                    transition,
                    position,
                },
            )
            .with_pointer_device_kind(UiHostPointerDeviceKind::Mouse)
            .unwrap()],
        })
        .unwrap();
        let crate::facade::interaction::UiHostInteractionIngressOutcome::Applied(receipt) =
            world.session.admit_host_interaction_batch(batch)
        else {
            panic!("valid pointer input must be admitted")
        };
        events.push(receipt);
    }
    events
}
