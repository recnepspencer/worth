#[path = "integrated_appearance_world/portal_order_assertion.rs"]
mod portal_order_assertion;
use portal_order_assertion::assert_portal_backdrop_order;
#[path = "integrated_appearance_world/admitted_dismissal.rs"]
mod admitted_dismissal;
#[path = "integrated_appearance_world/authored.rs"]
mod authored;
#[path = "integrated_appearance_world/backdrop_role_succession.rs"]
mod backdrop_role_succession;
#[path = "integrated_appearance_world/captured_geometry.rs"]
mod captured_geometry;
#[path = "integrated_appearance_world/content_entry.rs"]
mod content_entry;
#[path = "integrated_appearance_world/content_recovery.rs"]
mod content_recovery;
#[path = "integrated_appearance_world/entrance_publication.rs"]
mod entrance_publication;
#[path = "integrated_appearance_world/focus_modality.rs"]
mod focus_modality;
#[path = "integrated_appearance_world/foreign_acknowledgement.rs"]
mod foreign_acknowledgement;
#[path = "integrated_appearance_world/geometry.rs"]
mod geometry;
#[path = "integrated_appearance_world/hostile_protocol.rs"]
mod hostile_protocol;
#[path = "integrated_appearance_world/input_completion_order.rs"]
mod input_completion_order;
#[path = "integrated_appearance_world/interleaving_model.rs"]
mod interleaving_model;
#[path = "integrated_appearance_world/locality.rs"]
mod locality;
#[path = "integrated_appearance_world/modal_pointer_shielding.rs"]
mod modal_pointer_shielding;
#[path = "integrated_appearance_world/motion.rs"]
mod motion;
#[path = "integrated_appearance_world/motion_reconstruction.rs"]
mod motion_reconstruction;
#[path = "integrated_appearance_world/motion_work.rs"]
mod motion_work;
#[path = "integrated_appearance_world/native_attribution.rs"]
mod native_attribution;
#[path = "integrated_appearance_world/palette.rs"]
mod palette;
#[path = "integrated_appearance_world/pointer_geometry.rs"]
mod pointer_geometry;
#[path = "integrated_appearance_world/portal_content_succession.rs"]
mod portal_content_succession;
#[path = "integrated_appearance_world/portal_dismissal_epoch.rs"]
mod portal_dismissal_epoch;
#[path = "integrated_appearance_world/portal_exit_retention.rs"]
mod portal_exit_retention;
#[path = "integrated_appearance_world/portal_lifecycle.rs"]
mod portal_lifecycle;
#[path = "integrated_appearance_world/portal_placement_succession.rs"]
mod portal_placement_succession;
#[path = "integrated_appearance_world/portal_scroll_region.rs"]
mod portal_scroll_region;
#[path = "integrated_appearance_world/portal_scroll_settle.rs"]
mod portal_scroll_settle;
#[path = "integrated_appearance_world/reconstruction.rs"]
mod reconstruction;
#[path = "integrated_appearance_world/replacement.rs"]
mod replacement;
#[path = "integrated_appearance_world/replacement_geometry.rs"]
mod replacement_geometry;
#[path = "integrated_appearance_world/scroll_addressed_owner.rs"]
mod scroll_addressed_owner;
#[path = "integrated_appearance_world/scroll_capture_cancellation.rs"]
mod scroll_capture_cancellation;
#[path = "integrated_appearance_world/scroll_chrome_fixture.rs"]
mod scroll_chrome_fixture;
#[path = "integrated_appearance_world/scroll_chrome_layout_rejection.rs"]
mod scroll_chrome_layout_rejection;
#[path = "integrated_appearance_world/scroll_coarse_wheel.rs"]
mod scroll_coarse_wheel;
#[path = "integrated_appearance_world/scroll_content_anchoring.rs"]
mod scroll_content_anchoring;
#[path = "integrated_appearance_world/scroll_direct_acceptance.rs"]
mod scroll_direct_acceptance;
#[path = "integrated_appearance_world/scroll_direct_resize.rs"]
mod scroll_direct_resize;
#[path = "integrated_appearance_world/scroll_empty_sample_acceptance.rs"]
mod scroll_empty_sample_acceptance;
#[path = "integrated_appearance_world/scroll_extent_acceptance.rs"]
mod scroll_extent_acceptance;
#[path = "integrated_appearance_world/scroll_gesture_latching.rs"]
mod scroll_gesture_latching;
#[path = "integrated_appearance_world/scroll_hover_reresolution.rs"]
mod scroll_hover_reresolution;
#[path = "integrated_appearance_world/scroll_input_burst.rs"]
mod scroll_input_burst;
#[path = "integrated_appearance_world/scroll_locality_narrowing.rs"]
mod scroll_locality_narrowing;
#[path = "integrated_appearance_world/scroll_modality_cancellation.rs"]
mod scroll_modality_cancellation;
#[path = "integrated_appearance_world/scroll_nested_settle.rs"]
mod scroll_nested_settle;
#[path = "integrated_appearance_world/scroll_pose_authority.rs"]
mod scroll_pose_authority;
#[path = "integrated_appearance_world/scroll_presentation_snapping.rs"]
mod scroll_presentation_snapping;
#[path = "integrated_appearance_world/scroll_reconstruction.rs"]
mod scroll_reconstruction;
#[path = "integrated_appearance_world/scroll_reduced_motion.rs"]
mod scroll_reduced_motion;
#[path = "integrated_appearance_world/scroll_sample_acceptance.rs"]
mod scroll_sample_acceptance;
#[path = "integrated_appearance_world/scroll_settle_beside_attempt.rs"]
mod scroll_settle_beside_attempt;
#[path = "integrated_appearance_world/scroll_settle_commit.rs"]
mod scroll_settle_commit;
#[path = "integrated_appearance_world/scroll_settle_displacement.rs"]
mod scroll_settle_displacement;
#[path = "integrated_appearance_world/scroll_settle_frame.rs"]
mod scroll_settle_frame;
#[path = "integrated_appearance_world/scroll_settle_hit_lead.rs"]
mod scroll_settle_hit_lead;
#[path = "integrated_appearance_world/scroll_settle_over_open_attempt.rs"]
mod scroll_settle_over_open_attempt;
#[path = "integrated_appearance_world/scroll_settle_rebind.rs"]
mod scroll_settle_rebind;
#[path = "integrated_appearance_world/scroll_settlement_lifecycle.rs"]
mod scroll_settlement_lifecycle;
#[path = "integrated_appearance_world/seam.rs"]
mod seam;
#[path = "integrated_appearance_world/services.rs"]
mod services;
#[path = "integrated_appearance_world/session.rs"]
mod session;
#[path = "integrated_appearance_world/session_publication.rs"]
mod session_publication;
#[path = "integrated_appearance_world/stationary_motion.rs"]
mod stationary_motion;
#[path = "integrated_appearance_world/surface_continuity.rs"]
mod surface_continuity;
#[path = "integrated_appearance_world/theme_switch.rs"]
mod theme_switch;
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
    worth_ui_host_headless::translate_appearance_projection_for_certification(&output).unwrap();
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
    assert_portal_backdrop_order(output, &world);
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

fn assert_always_region_backdrop(output: &UiUnpublishedAppearanceFrameProjection, world: &World) {
    let transcript =
        worth_ui_host_headless::translate_appearance_projection_for_certification(output).unwrap();
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
