use super::session::World;
use crate::runtime::interaction::gesture::UiPressedAppearanceClass;
use worth_ui_host_contract::*;

pub(super) fn move_and_stage_restored_target(world: &mut World, first_sequence: u64) {
    press_target(world, first_sequence);
    let regions = super::hostile_protocol::regions(world);
    super::geometry::install_moved_target(
        &mut world.session,
        world.surfaces,
        world.instances,
        regions,
    );
    let moved = world.prepare_surface(world.surfaces[0]);
    assert_geometry_damage(
        &moved,
        world.instances[1],
        super::geometry::MOVED_TARGET_BOX,
        true,
    );
    world.publish(moved, 9, true);
    assert_capture(world, UiPressedAppearanceClass::CapturedOutside);

    let regions = super::hostile_protocol::regions(world);
    super::geometry::install_restored_target(
        &mut world.session,
        world.surfaces,
        world.instances,
        regions,
    );
}

pub(super) fn assert_restored_portal_selection(frame: &crate::mounting::UiPreparedMountedFrame) {
    assert_eq!(
        frame
            .appearance_selection_cost_report()
            .selected_instance_count(),
        3,
        "restored geometry, captured Pressed, and Portal select their combined exact scope"
    );
}

pub(super) fn assert_restored_geometry_damage(world: &World) {
    let output = world
        .session
        .mounted
        .current_unpublished_appearance()
        .unwrap()
        .expect("accepted combined Portal frame retains its appearance output");
    assert_geometry_output(output, world.instances[1], super::geometry::BOXES[1], false);
}

pub(super) fn assert_restored_capture(world: &World) {
    assert_capture(world, UiPressedAppearanceClass::ArmedInside);
}

pub(super) fn assert_moved_capture(world: &World) {
    assert_capture(world, UiPressedAppearanceClass::CapturedOutside);
}

pub(super) fn evaluate_target_operability(world: &mut World, first_sequence: u64) {
    press_target(world, first_sequence);
    release_target_and_evaluate(world, first_sequence + 1);
}

pub(super) fn press_target(world: &mut World, sequence: u64) {
    let presentation = world
        .session
        .mounted
        .current_presentation_for_surface(world.surfaces[0])
        .unwrap();
    let batch = super::stationary_motion::inputs::pointer_batch(
        world.session.host_session.identity().as_u64(),
        presentation,
        sequence,
        UiHostPointerIdentity::new(7),
        target_position(),
        Some(UiHostPointerButtonTransition::Pressed),
        true,
    );
    let outcome = world.session.admit_host_interaction_batch(batch);
    let crate::facade::interaction::UiHostInteractionIngressOutcome::Applied(receipt) = outcome
    else {
        panic!("AP-07 pointer press admits: {outcome:?}")
    };
    assert!(receipt.pointer_presence_denials().is_empty());
    let pressed = world.session.interaction.pressed_appearance_snapshot();
    let posture = pressed
        .postures()
        .iter()
        .find(|posture| posture.pointer() == UiHostPointerIdentity::new(7))
        .unwrap();
    assert_eq!(posture.target(), world.instances[1]);
}

pub(super) fn release_target_and_evaluate(world: &mut World, sequence: u64) {
    let presentation = world
        .session
        .mounted
        .current_presentation_for_surface(world.surfaces[0])
        .unwrap();
    let batch = super::stationary_motion::inputs::pointer_batch(
        world.session.host_session.identity().as_u64(),
        presentation,
        sequence,
        UiHostPointerIdentity::new(7),
        target_position(),
        Some(UiHostPointerButtonTransition::Released),
        false,
    );
    let crate::facade::interaction::UiHostInteractionIngressOutcome::Applied(receipt) =
        world.session.admit_host_interaction_batch(batch)
    else {
        panic!("AP-07 captured pointer release admits")
    };
    let semantic = receipt
        .into_transitions()
        .into_vec()
        .into_iter()
        .find_map(|transition| match transition {
            crate::facade::interaction::UiInteractionTransition::Semantic(value) => Some(value),
            _ => None,
        });
    let interaction = semantic.expect("press/release produces the authored activation interaction");
    assert_eq!(interaction.target().mounted_instance(), world.instances[1]);
    let crate::facade::intent::UiIntentRouteResolution::Product(route) = world
        .session
        .resolve_intent_route(
            crate::facade::interaction::UiIntentRouteSource::mounted_interaction(interaction),
        )
        .unwrap()
    else {
        panic!("AP-07 activation interaction resolves its product route")
    };
    let candidate = world.session.prepare_intent_payload(route).unwrap();
    assert!(matches!(
        world.session.evaluate_intent_operability(candidate),
        crate::facade::intent::UiIntentOperabilityOutcome::Operable(_)
    ));
}

fn target_position() -> UiHostSurfacePosition {
    let [x, y, width, height] = super::geometry::BOXES[1];
    UiHostSurfacePosition::viewport_logical(
        ((x + width / 2.0) * 1_000.0) as i64,
        ((y + height / 2.0) * 1_000.0) as i64,
    )
}

fn assert_capture(
    world: &World,
    expected: crate::runtime::interaction::gesture::UiPressedAppearanceClass,
) {
    let current = world
        .session
        .mounted
        .current_presentation_for_surface(world.surfaces[0])
        .unwrap();
    let receipt = world
        .session
        .mounted
        .current_presented_incarnation_receipt(
            crate::mounting::UiMountedIncarnationAffinityInput {
                surface: world.surfaces[0],
                binding: current.binding(),
                mounted_instance: world.instances[1],
            },
            current,
        )
        .unwrap();
    let pressed = world.session.interaction.pressed_appearance_snapshot();
    let posture = pressed
        .postures()
        .iter()
        .find(|posture| posture.pointer() == UiHostPointerIdentity::new(7))
        .unwrap();
    assert_eq!(posture.target(), world.instances[1]);
    assert_eq!(posture.presentation(), current);
    assert_eq!(posture.node_receipt(), receipt);
    assert_eq!(posture.class(), expected);
}

fn assert_geometry_damage(
    frame: &crate::mounting::UiPreparedMountedFrame,
    target: UiMountedInstanceIdentity,
    expected: [f32; 4],
    target_local: bool,
) {
    if target_local {
        assert_eq!(
            frame
                .appearance_selection_cost_report()
                .selected_instance_count(),
            1,
            "presented hit refresh selects only the captured semantic consumer"
        );
    }
    let profile = worth_ui_host_native::appearance_capability_report();
    let output =
        frame.lower_unpublished_appearance_with_profile_for_test(profile.appearance_profile());
    assert_geometry_output(&output, target, expected, target_local);
}

fn assert_geometry_output(
    output: &UiUnpublishedAppearanceFrameProjection,
    target: UiMountedInstanceIdentity,
    expected: [f32; 4],
    target_local: bool,
) {
    let nodes = output
        .fragments()
        .iter()
        .filter_map(|fragment| match fragment.identity() {
            UiUnpublishedAppearanceFragmentIdentity::NodeReceipt {
                successor: Some(receipt),
                ..
            } => Some((receipt, fragment)),
            _ => None,
        })
        .collect::<Vec<_>>();
    if target_local {
        assert!(
            nodes.iter().all(|(receipt, fragment)| {
                receipt.mounted_instance() == target
                    || (fragment.work().changes().is_empty() && fragment.work().damage().is_empty())
            }),
            "captured geometry work stays target-local"
        );
    }
    let target_fragment = nodes
        .iter()
        .find_map(|(receipt, fragment)| (receipt.mounted_instance() == target).then_some(*fragment))
        .expect("the moved captured target owns exact retained work");
    assert!(
        !target_fragment.work().damage().is_empty(),
        "a presented geometry move carries physical damage"
    );
    let surface = target_fragment
        .work()
        .successor()
        .mechanics()
        .iter()
        .find_map(|mechanic| match mechanic {
            UiMountedAppearanceMechanic::Surface(surface) => Some(surface),
            _ => None,
        })
        .expect("the six-axis target keeps its mounted surface mechanic");
    let bounds = surface.bounds();
    assert_eq!(
        [
            bounds.x(),
            bounds.y(),
            bounds.width() as i32,
            bounds.height() as i32
        ],
        [
            (expected[0] * 1_000.0) as i32,
            (expected[1] * 1_000.0) as i32,
            (expected[2] * 1_000.0) as i32,
            (expected[3] * 1_000.0) as i32,
        ]
    );
}
