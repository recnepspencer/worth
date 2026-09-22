use super::*;
use crate::native::presentation::raster::{raster_physical_bounds, UiNativeRasterBasis};
use crate::native::presentation::retained_draw_list::UiNativeRetainedMutationCounters;
use worth_ui_host_contract::{UiMountedPresentationSample, UiMountedPresentationSampleInput};

#[test]
fn moved_appearance_lookup_is_local_and_rollback_restores_exact_coverage() {
    for count in [64, 512] {
        let world = DrawListWorld::new();
        let frame = worth_ui_host_contract::UiMountedFrameIdentity::mint_unbound().unwrap();
        let mut retained = UiNativeRetainedDrawList::from_complete(
            frame,
            world.surface,
            world.binding,
            world.content,
            world.requirement.baseline(),
            &[],
            &[],
            UiMountedPaintOrderIntegrity::for_order(&[]),
            &[],
        )
        .unwrap();
        let mut appearance =
            UiNativeAppearanceRetained::new(UiNativeAppearanceScale::qualified(1_000).unwrap());
        let mut predecessor = None;
        let mut changes = Vec::new();
        for index in 0..count {
            let row = world.rect_at_order(
                frame,
                UiMountedInstanceIdentity::mint_unbound().unwrap(),
                (index * 40) as f32,
                UiMountedRgba8::new(20, 30, 40, 255),
                index as u32,
            );
            predecessor = Some(
                appearance
                    .insert(
                        UiNativeAppearanceCommand::Surface(surface_at(row, index as u32)),
                        predecessor,
                    )
                    .unwrap(),
            );
            let source = row.bounds();
            let moved = UiMountedCanonicalBox::canonicalize(UiMountedCanonicalBoxInput {
                x: source.x(),
                y: 40.0,
                width: source.width(),
                height: source.height(),
                coordinate_space: UiMountedCoordinateSpace::Viewport,
            })
            .unwrap();
            changes.push(UiMountedPresentationSampleChange::from_runtime_sampling(
                UiMountedPaintCommandIdentity::appearance_surface(row.owner()),
                Some(UiMountedPresentationTransform::from_runtime_sampling(source, moved).unwrap()),
                UiMountedPresentationOpacity::from_runtime_composition(u16::MAX),
            ));
        }
        retained.staged_appearance = Some((world.requirement, appearance));
        let sample = |changes| {
            UiMountedPresentationSample::from_inert_mechanics(UiMountedPresentationSampleInput {
                frame,
                surface: world.surface,
                binding: world.binding,
                content: world.content,
                baseline: world.requirement.baseline(),
                production_cost: Default::default(),
                changes,
                damage: Vec::new(),
            })
            .unwrap()
        };
        let initial = sample(changes.clone());
        let (_, initial_undo) = retained.stage_sample(&initial).unwrap();
        let tail = *changes.last().unwrap();
        let source = tail.transform().unwrap().source();
        let shifted = UiMountedCanonicalBox::canonicalize(UiMountedCanonicalBoxInput {
            x: source.x(),
            y: 80.0,
            width: source.width(),
            height: source.height(),
            coordinate_space: UiMountedCoordinateSpace::Viewport,
        })
        .unwrap();
        let next = sample(vec![
            UiMountedPresentationSampleChange::from_runtime_sampling(
                tail.command(),
                Some(
                    UiMountedPresentationTransform::from_runtime_sampling(source, shifted).unwrap(),
                ),
                tail.opacity(),
            ),
        ]);
        let (_, undo) = retained.stage_sample(&next).unwrap();
        let extent = [count as u32 * 40, 120];
        let basis = UiNativeRasterBasis::new(extent, 1.0);
        let atlas = crate::native::text_atlas::UiNativeTextAtlas::new();
        let query = |retained: &mut UiNativeRetainedDrawList, y| {
            let left = (count as u32 - 1) * 40;
            let clear = raster_physical_bounds([left, y, left + 32, y + 24], extent).unwrap();
            let mut counters = UiNativeRetainedMutationCounters::default();
            let operations = retained
                .appearance_operations_for_damage(clear, basis, &[], &atlas, &mut counters)
                .unwrap();
            assert!(
                counters.damage_index_branch_aabb_probes < 64,
                "{counters:?}"
            );
            assert!(
                counters.damage_index_leaf_command_bounds_probes <= 2,
                "{counters:?}"
            );
            assert_eq!(counters.retained_command_scans, 0);
            operations.len()
        };
        assert_eq!(query(&mut retained, 80), 1);
        assert_eq!(query(&mut retained, 40), 0);
        retained.rollback_sample(undo).unwrap();
        assert_eq!(query(&mut retained, 80), 0);
        assert_eq!(query(&mut retained, 40), 1);
        retained.reconstruct_sampled_appearance_coverage().unwrap();
        assert_eq!(query(&mut retained, 40), 1);
        retained.rollback_sample(initial_undo).unwrap();
        assert_eq!(query(&mut retained, 40), 0);
        assert_eq!(query(&mut retained, 0), 1);
    }
}
