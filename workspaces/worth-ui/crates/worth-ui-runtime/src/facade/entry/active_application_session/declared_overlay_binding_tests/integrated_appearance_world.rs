#[path = "integrated_appearance_world/authored.rs"]
mod authored;
#[path = "integrated_appearance_world/geometry.rs"]
mod geometry;
#[path = "integrated_appearance_world/locality.rs"]
mod locality;
#[path = "integrated_appearance_world/motion.rs"]
mod motion;
#[path = "integrated_appearance_world/motion_reconstruction.rs"]
mod motion_reconstruction;
#[path = "integrated_appearance_world/palette.rs"]
mod palette;
#[path = "integrated_appearance_world/portal_lifecycle.rs"]
mod portal_lifecycle;
#[path = "integrated_appearance_world/reconstruction.rs"]
mod reconstruction;
#[path = "integrated_appearance_world/seam.rs"]
mod seam;
#[path = "integrated_appearance_world/services.rs"]
mod services;
#[path = "integrated_appearance_world/session.rs"]
mod session;
use session::World;
use worth_ui_host_contract::*;

#[test]
fn shared_authored_world_preserves_surface_outline_and_original_range_text() {
    let mut world = World::launch();
    let frame = world.prepare();
    let profile = worth_ui_host_native::appearance_capability_report();
    let output =
        frame.lower_unpublished_appearance_with_profile_for_test(profile.appearance_profile());
    assert_eq!(output.fragments().len(), 4);
    let adopted = authored::text_contract().scalar_spans()[0].paint_identity();
    for (index, target) in world.instances[..4].iter().enumerate() {
        let fragment = output.fragments().iter().find(|fragment| {
            matches!(fragment.identity(), UiUnpublishedAppearanceFragmentIdentity::NodeReceipt { successor: Some(receipt), .. } if receipt.mounted_instance() == *target)
        }).unwrap();
        let mechanics = fragment.work().successor().mechanics();
        assert_eq!(mechanics.len(), 3);
        let mut families = [false; 3];
        for mechanic in mechanics {
            match mechanic {
                UiMountedAppearanceMechanic::Surface(surface) => {
                    families[0] = true;
                    assert_eq!(surface.node_receipt().mounted_instance(), *target);
                    assert_eq!(surface.opacity().units(), 40_000);
                    assert_eq!(
                        surface.surface_paint_order(),
                        authored::PAINT_ORDERS[if index == 3 { 0 } else { index }]
                    );
                    if index == 3 {
                        let clip = surface.clip();
                        assert_eq!(
                            (clip.x(), clip.y(), clip.width(), clip.height()),
                            (50_000, 60_000, 120_000, 40_000),
                            "the secondary Mosaic occurrence carries its exact region clip"
                        );
                    }
                }
                UiMountedAppearanceMechanic::Outline(outline) => {
                    families[1] = true;
                    assert_eq!(outline.node_receipt().mounted_instance(), *target);
                    assert_eq!(outline.opacity().units(), 40_000);
                }
                UiMountedAppearanceMechanic::TextForeground(text) => {
                    families[2] = true;
                    assert_eq!(text.node_receipt().mounted_instance(), *target);
                    assert_eq!(text.paint_span().digest(), adopted);
                    assert_eq!(text.opacity().units(), 40_000);
                }
                other => panic!("unexpected ordinary mechanic {other:?}"),
            }
        }
        assert_eq!(families, [true; 3]);
        let candidate = fragment
            .text_candidates()
            .iter()
            .find(|text| text.text() == "AB")
            .unwrap();
        assert_eq!(candidate.mounted_instance(), *target);
        if index == 3 {
            assert_eq!(
                candidate.clip_bounds(),
                geometry::viewport(geometry::SECONDARY_REGION),
                "text consumes the same occurrence-local Mosaic region"
            );
        }
        assert_eq!(
            candidate.foregrounds()[0].original_range(),
            UiTextOriginalRange::new(0, 1).unwrap()
        );
        assert_eq!(
            candidate.foregrounds()[1].original_range(),
            UiTextOriginalRange::new(1, 2).unwrap()
        );
    }
    worth_ui_host_headless::translate_unpublished_appearance_for_certification(&output).unwrap();
    world.publish(frame, 1, true);
    let initial = world
        .session
        .mounted
        .current_unpublished_appearance()
        .unwrap()
        .unwrap();
    assert_always_region_backdrop(initial, &world);
    locality::appearance_changes(&mut world);
    let unchanged = world.prepare_surface(world.surfaces[0]);
    unchanged.assert_no_unpublished_appearance_for_test();
    drop(unchanged);
    let restoration_target = world.focus_first_surface_participant();
    let first = world.open(0, "overlay.menu", None, 10);
    let second = world.open(1, "overlay.menu", None, 11);
    let child = world.open(2, "overlay.child", Some(first), 12);
    let overlays = world.prepare_surface_with_current_portals(world.surfaces[0]);
    let commands = overlays
        .surfaces()
        .iter()
        .flat_map(|surface| surface.projection().retained_paint_commands().to_vec())
        .collect::<Vec<_>>();
    assert!(commands.iter().any(|command| matches!(command, UiMountedPaintCommand::SemanticText { mechanic, .. } if mechanic.mounted_instance() == world.instances[4])), "Portal content has a real retained text command");
    world.publish(overlays, 30, false);
    let output = world
        .session
        .mounted
        .current_unpublished_appearance()
        .unwrap()
        .unwrap();
    assert!(output.fragments().iter().any(|fragment| fragment.identity()
        == UiUnpublishedAppearanceFragmentIdentity::SurfaceOverlay(world.surfaces[0])));
    assert_portal_backdrop_order(&output, &world);
    motion::compose_declared_samples(&mut world);
    world.close_parent(first, child, second, restoration_target, 200);
    reconstruction::cold_surface(&mut world);
    let _ = world.session.shutdown();
}

#[test]
fn disjoint_mounted_region_neighborhood_suppresses_child_appearance() {
    let mut world = World::launch();
    let bindings = world
        .session
        .application
        .authored_overlay_material()
        .overlay_declaration_bindings();
    let declarations =
        ["workspace.surface.overlay", "workspace.surface.secondary"].map(|surface| {
            bindings
                .region_named(surface, "workspace.region.primary")
                .unwrap()
        });
    geometry::install_disjoint_child_region(
        &mut world.session,
        world.surfaces,
        world.instances,
        declarations,
    );
    let target = world.instances[4];
    let frame = world.prepare_surface(world.surfaces[0]);
    assert_eq!(
        frame
            .projection_rc_for_test()
            .appearance_clip_for_test(target),
        Some(crate::mounting::UiMountedAppearanceClip::Suppressed),
        "the child and parent region occurrences complete to disjoint ancestor clips",
    );
    let profile = worth_ui_host_native::appearance_capability_report();
    let output =
        frame.lower_unpublished_appearance_with_profile_for_test(profile.appearance_profile());
    assert!(output
        .fragments()
        .iter()
        .flat_map(|fragment| fragment.work().successor().mechanics())
        .all(|mechanic| match mechanic {
            UiMountedAppearanceMechanic::Surface(row) => {
                row.node_receipt().mounted_instance() != target
            }
            UiMountedAppearanceMechanic::Outline(row) => {
                row.node_receipt().mounted_instance() != target
            }
            UiMountedAppearanceMechanic::TextForeground(row) => {
                row.node_receipt().mounted_instance() != target
            }
            _ => true,
        }));
    let _ = world.session.shutdown();
}

fn assert_always_region_backdrop(output: &UiUnpublishedAppearanceFrameProjection, world: &World) {
    let transcript =
        worth_ui_host_headless::translate_unpublished_appearance_for_certification(output).unwrap();
    let overlay = transcript
        .fragments()
        .iter()
        .find(|fragment| {
            fragment.identity()
                == UiUnpublishedAppearanceFragmentIdentity::SurfaceOverlay(world.surfaces[1])
        })
        .expect("Always Backdrop publishes without any Portal on its surface");
    let [worth_ui_host_headless::UiHeadlessAppearanceMechanic::Backdrop(backdrop)] =
        overlay.work().successor().mechanics()
    else {
        panic!("the secondary overlay contains only its Always Backdrop");
    };
    assert_eq!(
        backdrop.identity().declaration_projection(),
        backdrop_projection(world, "overlay.ambient")
    );
    let extent = backdrop.extent();
    assert_eq!(
        (extent.x(), extent.y(), extent.width(), extent.height()),
        (50_000, 60_000, 120_000, 40_000)
    );
    assert_eq!(
        overlay
            .work()
            .successor()
            .reference_overlay_at(60_000, 70_000)
            .straight_srgba(),
        [4, 8, 12, 128]
    );
    assert_eq!(
        overlay
            .work()
            .successor()
            .reference_overlay_at(40_000, 50_000)
            .straight_srgba(),
        [0, 0, 0, 0]
    );
}

fn assert_portal_backdrop_order(output: &UiUnpublishedAppearanceFrameProjection, world: &World) {
    let overlay = output
        .fragments()
        .iter()
        .find(|fragment| {
            fragment.identity()
                == UiUnpublishedAppearanceFragmentIdentity::SurfaceOverlay(world.surfaces[0])
        })
        .unwrap();
    let order = overlay.work().successor().overlay_order().bottom_to_top();
    for portal in world.instances[..2].iter().copied() {
        let before = overlay
            .work()
            .successor()
            .mechanics()
            .iter()
            .find_map(|mechanic| match mechanic {
                UiMountedAppearanceMechanic::Backdrop(backdrop)
                    if backdrop.identity().declaration_projection()
                        == backdrop_projection(world, "overlay.scrim")
                        && backdrop.identity().scope()
                            == UiMountedBackdropScope::PerPortalInstance(portal) =>
                {
                    Some(backdrop.identity().clone())
                }
                _ => None,
            })
            .unwrap_or_else(|| {
                panic!(
                    "missing before Backdrop for {portal:?}; mechanics: {:?}",
                    overlay.work().successor().mechanics()
                )
            });
        let after = overlay
            .work()
            .successor()
            .mechanics()
            .iter()
            .find_map(|mechanic| match mechanic {
                UiMountedAppearanceMechanic::Backdrop(backdrop)
                    if backdrop.identity().declaration_projection()
                        == backdrop_projection(world, "overlay.after")
                        && backdrop.identity().scope()
                            == UiMountedBackdropScope::PerPortalInstance(portal) =>
                {
                    Some(backdrop.identity().clone())
                }
                _ => None,
            })
            .unwrap_or_else(|| {
                panic!(
                    "missing after Backdrop for {portal:?}; mechanics: {:?}",
                    overlay.work().successor().mechanics()
                )
            });
        let position = |participant: &UiOverlayParticipantIdentity| {
            order
                .iter()
                .position(|candidate| candidate == participant)
                .unwrap()
        };
        let before = position(&UiOverlayParticipantIdentity::Backdrop(before));
        let portal = position(&UiOverlayParticipantIdentity::Portal(portal));
        let after = position(&UiOverlayParticipantIdentity::Backdrop(after));
        assert_eq!((before + 1, portal + 1), (portal, after));
    }
    assert!(overlay
        .work()
        .successor()
        .mechanics()
        .iter()
        .all(|mechanic| {
            !matches!(mechanic, UiMountedAppearanceMechanic::Backdrop(backdrop)
            if backdrop.identity().scope()
                == UiMountedBackdropScope::PerPortalInstance(world.instances[2]))
        }));
}

fn backdrop_projection(world: &World, name: &str) -> String {
    let declaration = world
        .session
        .application
        .authored_overlay_material()
        .overlay_declaration_bindings()
        .backdrop_named(name)
        .unwrap();
    format!("backdrop:{}", declaration.value())
}
