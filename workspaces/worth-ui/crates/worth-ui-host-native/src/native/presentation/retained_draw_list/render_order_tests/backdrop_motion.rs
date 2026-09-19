use super::*;
use worth_ui_host_contract::*;

#[test]
fn fast_samples_restore_backdrop_outside_body_and_keep_exact_portal_affinity() {
    for include_other in [false, true] {
        let world = DrawListWorld::new();
        let frame = UiMountedFrameIdentity::mint_unbound().unwrap();
        let portal = world.rect(frame, world.first, 10.0, UiMountedRgba8::new(1, 2, 3, 255));
        let other_bounds = world
            .rect(frame, world.first, 50.0, UiMountedRgba8::new(1, 2, 3, 255))
            .bounds();
        let other = UiMountedPortalOverlayMechanic::complete_from_runtime_mounting(
            UiMountedPortalOverlayCompletionInput {
                frame,
                surface: world.surface,
                binding: world.binding,
                owner: world.first,
                owner_receipt: portal.owner_receipt(),
                portal_identity: portal.portal_identity() + 1,
                anchor_presentation: portal.anchor_presentation(),
                anchor_bounds: portal.anchor_bounds(),
                bounds: other_bounds,
                paint_bounds: other_bounds,
                clip_bounds: other_bounds,
                color: portal.color(),
                layer_semantic_order: portal.layer_semantic_order(),
                layer_depth: portal.layer_depth(),
                lifecycle: portal.lifecycle(),
                shielding: portal.shielding(),
            },
        )
        .unwrap();
        let commands = [portal, other]
            .into_iter()
            .take(if include_other { 2 } else { 1 })
            .map(|mechanic| UiMountedPaintCommand::PortalOverlay {
                identity: UiMountedPaintCommandIdentity::portal_overlay(&mechanic),
                mechanic,
            })
            .collect::<Vec<_>>();
        let order = commands
            .iter()
            .map(|command| UiMountedPaintOrderIdentity::for_command(command.identity()))
            .collect::<Vec<_>>();
        let mut retained = UiNativeRetainedDrawList::from_complete(
            frame,
            world.surface,
            world.binding,
            world.content,
            world.requirement.baseline(),
            &commands,
            &order,
            UiMountedPaintOrderIntegrity::for_order(&order),
            &[],
        )
        .unwrap();
        let placement = UiOverlayPlacementReceipt::from_runtime_overlay_order(1, 0).unwrap();
        let backdrop = UiMountedBackdropMechanic::complete_from_runtime_mounting(
            UiMountedBackdropCompletionInput {
                identity: UiMountedBackdropIdentity::from_runtime_mounting(
                    "review.scrim",
                    UiMountedBackdropScope::SurfaceSingleton(world.surface),
                    1,
                )
                .unwrap(),
                semantic_surface: world.surface,
                placement,
                extent: UiAppearanceBackdropExtent::new(0, 0, 100_000, 100_000).unwrap(),
                clip: UiAppearanceClip::new(0, 0, 100_000, 100_000).unwrap(),
                background: UiMountedAppearanceColor::from_straight_srgba([0, 0, 0, 255]),
                opacity: UiMountedPresentationOpacity::from_runtime_appearance_motion(
                    UiMountedAppearanceOpacity::from_units(32_768),
                    0,
                ),
                motion_target: Some(UiMountedPortalPresentationAffinity::from_runtime_mounting(
                    world.first,
                    portal.portal_identity(),
                )),
                attribution: UiMountedBackdropAppearanceAttribution::from_runtime_transport(
                    world.surface,
                    placement,
                    1,
                    1,
                )
                .unwrap(),
            },
        )
        .unwrap();
        let mut appearance =
            UiNativeAppearanceRetained::new(UiNativeAppearanceScale::qualified(1_000).unwrap());
        let key = appearance
            .insert(UiNativeAppearanceCommand::Backdrop(backdrop.clone()), None)
            .unwrap();
        appearance
            .insert(
                UiNativeAppearanceCommand::OverlayOrder(
                    UiMountedOverlayOrderMechanic::complete_from_runtime_overlay_order(
                        world.surface,
                        UiMountedPresentationAttemptIdentity::mint_unbound().unwrap(),
                        1,
                        1,
                        [
                            UiOverlayParticipantIdentity::Backdrop(backdrop.identity().clone()),
                            UiOverlayParticipantIdentity::Portal(world.first),
                        ],
                    )
                    .unwrap(),
                ),
                Some(key),
            )
            .unwrap();
        retained.staged_appearance = Some((world.requirement, appearance));
        let basis = crate::native::presentation::raster::UiNativeRasterBasis::new([100, 100], 1.0);
        let atlas = crate::native::text_atlas::UiNativeTextAtlas::new();
        retained
            .initialize_physical_coverage(basis, &atlas)
            .unwrap();
        for (motion, alpha) in [(32_768, 64), (65_535, 128)] {
            let sample = UiMountedPresentationSample::from_inert_mechanics(
                UiMountedPresentationSampleInput {
                    frame,
                    surface: world.surface,
                    binding: world.binding,
                    content: world.content,
                    baseline: world.requirement.baseline(),
                    production_cost: Default::default(),
                    changes: commands
                        .iter()
                        .enumerate()
                        .map(|(index, command)| {
                            UiMountedPresentationSampleChange::from_runtime_sampling(
                                command.identity(),
                                None,
                                UiMountedPresentationOpacity::from_runtime_appearance_motion(
                                    UiMountedAppearanceOpacity::from_units(10_000),
                                    if index == 0 { motion } else { 0 },
                                ),
                            )
                        })
                        .collect(),
                    damage: vec![UiMountedLogicalDamage::from_runtime_mounting(
                        portal.bounds(),
                    )],
                },
            )
            .unwrap();
            let before = retained.backdrop_sample_opacity(&backdrop).unwrap();
            if include_other {
                // Isolate exact sampling selection. Full overlay presentation still
                // denies multiple Portal anchors for the same mounted owner.
                let (_, undo) = retained.stage_sample(&sample).unwrap();
                assert_eq!(
                    retained.backdrop_sample_opacity(&backdrop).unwrap().units(),
                    if motion == 32_768 { 16_384 } else { 32_768 }
                );
                retained.rollback_sample(undo).unwrap();
                assert_eq!(retained.backdrop_sample_opacity(&backdrop).unwrap(), before);
                continue;
            }
            let (plan, undo) = crate::native::presentation::sample::prepare_sample_plan(
                basis,
                &sample,
                &atlas,
                &mut retained,
            )
            .unwrap_or_else(|_| panic!("native sample preparation"));
            // (90,90) is outside the independently authored 32x24 Portal at (10,0).
            assert!(plan.operations.iter().any(|operation| matches!(operation,
            crate::native::presentation::UiNativeRasterOperation::FilledRect { rect, source_rgba8 }
            if rect.physical_bounds() == [0.0, 0.0, 100.0, 100.0] && *source_rgba8 == [0, 0, 0, alpha]
        )), "the backdrop's own half opacity follows only its named Portal, over the whole extent");
            retained.rollback_sample(undo).unwrap();
            assert_eq!(retained.backdrop_sample_opacity(&backdrop).unwrap(), before);
        }
        let (_, appearance) = retained.staged_appearance.as_mut().unwrap();
        let undo = appearance.stage_command_remove(key).unwrap();
        assert_eq!(
            appearance
                .backdrops_for_motion(backdrop.motion_target().unwrap())
                .count(),
            0
        );
        appearance.rollback_text(undo).unwrap();
        assert_eq!(
            appearance
                .backdrops_for_motion(backdrop.motion_target().unwrap())
                .count(),
            1
        );
        let target_b = UiMountedPortalPresentationAffinity::from_runtime_mounting(
            world.first,
            other.portal_identity(),
        );
        let rebound = UiMountedBackdropMechanic::complete_from_runtime_mounting(
            UiMountedBackdropCompletionInput {
                identity: backdrop.identity().clone(),
                semantic_surface: world.surface,
                placement,
                extent: backdrop.extent(),
                clip: backdrop.clip(),
                background: backdrop.background(),
                opacity: backdrop.opacity(),
                motion_target: Some(target_b),
                attribution: backdrop.attribution(),
            },
        )
        .unwrap();
        let undo = appearance
            .stage_command_replace(key, UiNativeAppearanceCommand::Backdrop(rebound))
            .unwrap();
        assert_eq!(
            appearance
                .backdrops_for_motion(backdrop.motion_target().unwrap())
                .count(),
            0
        );
        assert_eq!(appearance.backdrops_for_motion(target_b).count(), 1);
        appearance.rollback_text(undo).unwrap();
        assert_eq!(
            appearance
                .backdrops_for_motion(backdrop.motion_target().unwrap())
                .count(),
            1
        );
        assert_eq!(appearance.backdrops_for_motion(target_b).count(), 0);
    }
}
