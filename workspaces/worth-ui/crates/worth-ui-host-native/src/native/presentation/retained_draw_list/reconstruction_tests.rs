use worth_ui_host_contract::{
    UiMountedFrameIdentity, UiMountedLogicalDamage, UiMountedPaintCommandChange,
    UiMountedPresentationDelta, UiMountedPresentationDeltaInput, UiMountedPresentationOpacity,
    UiMountedPresentationReconstruction, UiMountedPresentationReconstructionInput,
    UiMountedPresentationSampleChange, UiMountedRgba8,
};

use super::{command, DrawListWorld};
use crate::native::presentation::UiNativeRetainedDrawList;

#[test]
fn cold_reconstruction_rebuilds_every_index_then_next_delta_remains_local() {
    let world = DrawListWorld::new();
    let predecessor = UiMountedFrameIdentity::mint_unbound().unwrap();
    let reconstructed_frame = UiMountedFrameIdentity::mint_unbound().unwrap();
    let rows = [
        world.rect(
            reconstructed_frame,
            world.first,
            0.0,
            UiMountedRgba8::new(10, 20, 30, 255),
        ),
        world.rect(
            reconstructed_frame,
            world.second,
            80.0,
            UiMountedRgba8::new(40, 50, 60, 255),
        ),
    ];
    let complete = world.initial(reconstructed_frame, rows);
    let reconstruction = UiMountedPresentationReconstruction::from_inert_mechanics(
        UiMountedPresentationReconstructionInput {
            predecessor,
            successor: reconstructed_frame,
            surface: world.surface,
            binding: world.binding,
            content: world.content,
            baseline: world.requirement.baseline(),
            projection: complete.projection().clone(),
            commands: complete.commands().to_vec(),
            sample_overrides: Vec::new(),
            order: complete.order().to_vec(),
            order_integrity: complete.order_integrity(),
            damage: complete.damage().to_vec(),
            production_cost: Default::default(),
        },
    );

    let mut retained = UiNativeRetainedDrawList::reconstruction(&reconstruction, &[]).unwrap();
    assert_eq!(retained.order.ordered().count(), 2);
    let full_query = retained.damage.intersecting(rows[0].bounds()).unwrap();
    assert_eq!(full_query.stored_records, 2);
    assert_eq!(full_query.high_water_records, 2);

    let successor = UiMountedFrameIdentity::mint_unbound().unwrap();
    let replacement = world.rect(
        successor,
        world.first,
        0.0,
        UiMountedRgba8::new(70, 80, 90, 255),
    );
    let replacement_command = command(replacement);
    let delta = UiMountedPresentationDelta::from_inert_mechanics(UiMountedPresentationDeltaInput {
        predecessor: reconstructed_frame,
        successor,
        surface: world.surface,
        binding: world.binding,
        content: world.content,
        baseline: world.requirement.baseline(),
        changes: vec![UiMountedPaintCommandChange::replacement(
            command(rows[0]).identity(),
            replacement_command.clone(),
        )],
        nodes: Vec::new(),
        order: Vec::new(),
        order_integrity: complete.order_integrity(),
        damage: vec![
            UiMountedLogicalDamage::from_runtime_mounting(rows[0].bounds()),
            UiMountedLogicalDamage::from_runtime_mounting(replacement.bounds()),
        ],
        auxiliary: None,
        production_cost: Default::default(),
    });
    let plan = retained.apply_delta(&delta).unwrap();
    assert_eq!(plan.counters.draw_mutations, 1);
    assert_eq!(plan.counters.order_mutations, 0);
    assert_eq!(plan.counters.retained_command_scans, 0);
    assert_eq!(plan.counters.damage_index_stored_records, 2);
    assert_eq!(plan.counters.damage_index_high_water, 2);
    assert_eq!(
        retained.command(replacement_command.identity()),
        Some(&replacement_command)
    );
}

#[test]
fn cold_reconstruction_installs_and_rasterizes_accepted_motion_override() {
    let world = DrawListWorld::new();
    let predecessor = UiMountedFrameIdentity::mint_unbound().unwrap();
    let frame = UiMountedFrameIdentity::mint_unbound().unwrap();
    let rect = world.rect(
        frame,
        world.first,
        10.0,
        UiMountedRgba8::new(30, 60, 90, 255),
    );
    let complete = world.initial(frame, [rect]);
    let identity = command(rect).identity();
    let override_change = UiMountedPresentationSampleChange::from_runtime_sampling(
        identity,
        None,
        UiMountedPresentationOpacity::from_runtime_composition(32_768),
    );
    let reconstruction = UiMountedPresentationReconstruction::from_inert_mechanics(
        UiMountedPresentationReconstructionInput {
            predecessor,
            successor: frame,
            surface: world.surface,
            binding: world.binding,
            content: world.content,
            baseline: world.requirement.baseline(),
            projection: complete.projection().clone(),
            commands: complete.commands().to_vec(),
            sample_overrides: vec![override_change],
            order: complete.order().to_vec(),
            order_integrity: complete.order_integrity(),
            damage: complete.damage().to_vec(),
            production_cost: Default::default(),
        },
    );
    let mut retained = UiNativeRetainedDrawList::reconstruction(&reconstruction, &[])
        .unwrap_or_else(|_| panic!("accepted reconstruction sample remains valid"));
    assert_eq!(retained.sample_override(identity), Some(override_change));
    let basis = crate::native::presentation::raster::UiNativeRasterBasis::new([100, 100], 1.0);
    let atlas = crate::native::text_atlas::UiNativeTextAtlas::new();
    retained
        .initialize_physical_coverage(basis, &atlas)
        .unwrap();
    let plan = crate::native::presentation::reconstruction::build_plan(basis, &atlas, &retained)
        .unwrap_or_else(|_| panic!("accepted reconstruction sample rasterizes"));
    assert!(plan.operations.iter().any(|operation| matches!(
        operation,
        crate::native::presentation::UiNativeRasterOperation::FilledRect {
            source_rgba8: [30, 60, 90, 128],
            ..
        }
    )));
}
