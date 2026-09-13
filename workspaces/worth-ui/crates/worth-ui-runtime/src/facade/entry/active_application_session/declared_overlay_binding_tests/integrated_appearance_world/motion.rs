use super::{locality, session::World};
use worth_ui_host_contract::*;

pub(super) fn compose_declared_samples(world: &mut World) {
    // The declared 140-tick entrance is cubic ease-out. At half duration its
    // raw u16 factor is round(65535 * 7/8) = 57343, not an 8-bit value.
    world.sample(world.surfaces[0], 1, 31, 0);
    composed(world, 0, &[], 32);
    world.sample(world.surfaces[0], 71, 33, 57_343);
    composed(world, 57_343, &[], 34);

    // The modal shields this background pointer move. The next accepted sample
    // must therefore remain physical-only, preserving the trigger's appearance.
    world.sample(world.surfaces[0], 141, 35, 65_535);
    locality::hover(world, 0, 4);
    composed(world, 65_535, &[], 35);
    let unchanged = world.prepare_surface_with_current_portals(world.surfaces[0]);
    assert_eq!(
        unchanged
            .appearance_selection_cost_report()
            .selected_instance_count(),
        0
    );
    world.publish(unchanged, 36, false);
    let unchanged_output = world
        .session
        .mounted
        .current_unpublished_appearance()
        .unwrap()
        .unwrap();
    assert!(unchanged_output.fragments().iter().all(|fragment| {
        fragment.work().changes().is_empty() && fragment.work().damage().is_empty()
    }));
}

fn composed(
    world: &mut World,
    motion: u16,
    appearance_targets: &[UiMountedInstanceIdentity],
    now: u64,
) {
    let frame = world.prepare_surface_with_current_portals(world.surfaces[0]);
    let content_commands = frame
        .surfaces()
        .iter()
        .flat_map(|surface| surface.projection().retained_paint_commands().to_vec())
        .filter(|command| {
            command.identity().mounted_instance() == world.instances[4]
                || matches!(command, UiMountedPaintCommand::PortalOverlay { .. })
        })
        .map(|command| command.identity())
        .collect::<Vec<_>>();
    assert!(!content_commands.is_empty());
    for command in &content_commands {
        let presentation = world
            .session
            .mounted
            .current_presentation_for_surface(world.surfaces[0])
            .unwrap();
        let accepted = world
            .session
            .mounted
            .accepted_motion_for_command(presentation, *command)
            .unwrap();
        assert_eq!(
            accepted.map(|sample| sample.opacity_units()),
            Some(motion),
            "content command accepted before successor admission"
        );
    }
    let invalidation = frame.appearance_invalidation_batch().unwrap();
    assert_eq!(
        frame
            .appearance_selection_cost_report()
            .selected_instance_count(),
        4
    );
    assert_eq!(
        invalidation.is_physical_input_only(),
        appearance_targets.is_empty()
    );
    if appearance_targets.is_empty() {
        world.publish(frame, now, false);
    } else {
        world.publish_in_flight(frame, now);
    }
    for command in content_commands {
        let presentation = world
            .session
            .mounted
            .current_presentation_for_surface(world.surfaces[0])
            .unwrap();
        let accepted = world
            .session
            .mounted
            .accepted_motion_for_command(presentation, command)
            .unwrap();
        assert_eq!(
            accepted.map(|sample| sample.opacity_units()),
            Some(motion),
            "command {command:?} inherited through successor admission"
        );
    }
    let output = world
        .session
        .mounted
        .current_unpublished_appearance()
        .unwrap()
        .unwrap();
    let mut targets = Vec::new();
    let mut backdrop_count = 0;
    let mut portal_count = 0;
    for fragment in output.fragments() {
        if let UiUnpublishedAppearanceFragmentIdentity::NodeReceipt {
            successor: Some(receipt),
            ..
        } = fragment.identity()
        {
            let target = receipt.mounted_instance();
            targets.push(target);
            let expected = if target == world.instances[4] {
                opacity(40_000, motion)
            } else {
                assert!(
                    world.instances[..3].contains(&target),
                    "another surface changed on Motion-only work"
                );
                40_000
            };
            let mut families = [false; 3];
            for mechanic in fragment.work().successor().mechanics() {
                match mechanic {
                    UiMountedAppearanceMechanic::Surface(surface) => {
                        families[0] = true;
                        assert_eq!(surface.opacity().units(), expected);
                        if let Some(index) = world.instances[..3]
                            .iter()
                            .position(|owner| *owner == target)
                        {
                            let [x, y, width, height] = super::geometry::BOXES[index];
                            let bounds = surface.bounds();
                            assert_eq!(
                                [
                                    bounds.x() as f32,
                                    bounds.y() as f32,
                                    bounds.width() as f32,
                                    bounds.height() as f32
                                ],
                                [x, y, width, height].map(|value| value * 1_000.0),
                                "Portal Motion cannot move or erase its ordinary anchor surface"
                            );
                            assert_eq!(surface.portal_group(), None);
                        }
                    }
                    UiMountedAppearanceMechanic::Outline(outline) => {
                        families[1] = true;
                        assert_eq!(outline.opacity().units(), expected);
                    }
                    UiMountedAppearanceMechanic::TextForeground(text) => {
                        families[2] = true;
                        assert_eq!(
                            text.opacity().units(),
                            expected,
                            "text command {:?}",
                            text.command()
                        );
                    }
                    other => panic!("unexpected node family {other:?}"),
                }
            }
            assert_eq!(families, [true, true, true]);
            assert!(fragment
                .text_candidates()
                .iter()
                .all(|candidate| candidate.performed_layout_cost().is_none()));
            if target == world.instances[4] {
                // Entrance publication already contains the zero-opacity sample.
                // Later samples change the retained ordinary appearance.
                assert_eq!(fragment.work().changes().is_empty(), motion == 0);
                assert_eq!(fragment.work().damage().is_empty(), motion == 0);
            } else if !appearance_targets.contains(&target) {
                assert!(
                    fragment.work().changes().is_empty(),
                    "stationary trigger {target:?} preserves all ordinary mechanics at tick {now}: changes={:?}, successor={:?}",
                    fragment.work().changes(),
                    fragment.work().successor().mechanics()
                );
            }
        }
        for mechanic in fragment.work().successor().mechanics() {
            match mechanic {
                UiMountedAppearanceMechanic::Backdrop(backdrop) => {
                    backdrop_count += 1;
                    assert_eq!(backdrop.opacity().units(), opacity(32_768, motion));
                }
                UiMountedAppearanceMechanic::PortalSurface(surface) => {
                    portal_count += 1;
                    assert_eq!(surface.surface().opacity().units(), opacity(40_000, motion));
                    let index = world.instances[..3]
                        .iter()
                        .position(|owner| *owner == surface.portal_instance())
                        .unwrap();
                    let bounds = surface.surface().bounds();
                    let expected = if index == 2 {
                        // The authored modal is 280x320, centered in 800x600.
                        [260_000, 140_000, 280_000, 320_000]
                    } else {
                        [
                            [40_000, 118_000, 280_000, 320_000],
                            [300_000, 128_000, 280_000, 320_000],
                        ][index]
                    };
                    assert_eq!(
                        [bounds.x(), bounds.y(), bounds.width() as i32, bounds.height() as i32],
                        expected,
                        "the Portal uses its owner-issued placement rather than the anchor allocation"
                    );
                }
                _ => {}
            }
        }
    }
    let mut expected = world.instances[..3].to_vec();
    expected.push(world.instances[4]);
    expected.sort_unstable();
    targets.sort_unstable();
    assert_eq!(targets, expected);
    assert_eq!((backdrop_count, portal_count), (4, 3));
    let transcript =
        worth_ui_host_headless::translate_appearance_projection_for_certification(output).unwrap();
    if motion == u16::MAX {
        let overlay = transcript
            .fragments()
            .iter()
            .find(|fragment| {
                fragment.identity()
                    == UiUnpublishedAppearanceFragmentIdentity::SurfaceOverlay(world.surfaces[0])
            })
            .unwrap();
        assert_eq!(
            overlay
                .work()
                .successor()
                .reference_overlay_at(750_000, 550_000)
                .straight_srgba(),
            [4, 8, 12, 191],
            "the two viewport Backdrops cover the point outside exact mounted regions"
        );
        // The anchor neighborhood contains three scrims, but no Portal surface.
        assert_eq!(
            overlay
                .work()
                .successor()
                .reference_overlay_at(310_000, 60_000)
                .straight_srgba(),
            [4, 8, 12, 223],
            "an ordinary anchor allocation cannot become a Portal overlay rectangle"
        );
        // The sibling dialog at [300,128..580,448] covers this point; neither
        // other dialog nor region Backdrop does. Two viewport scrims precede its
        // retained Hover color [192,64,32] with effective alpha
        // 128/255 * 40_000/65_535. Modal shielding blocked pointer departure.
        assert_eq!(
            overlay
                .work()
                .successor()
                .reference_overlay_at(560_000, 150_000)
                .straight_srgba(),
            [122, 39, 21, 211],
            "the sibling Portal paints its owner-issued dialog bounds"
        );
    }
}

fn opacity(appearance: u16, motion: u16) -> u16 {
    // Independent integer division with nearest-even rounding; never calls the
    // runtime composition operation or converts through host alpha bytes.
    let product = u64::from(appearance) * u64::from(motion);
    let quotient = product / 65_535;
    let remainder = product % 65_535;
    (quotient + u64::from(remainder * 2 > 65_535 || (remainder * 2 == 65_535 && quotient % 2 != 0)))
        as u16
}
