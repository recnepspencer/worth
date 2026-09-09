use super::prepare_delta_plan;
use crate::native::presentation::retained_raster::replay_cost;
use crate::native::presentation::{
    raster::UiNativeRasterBasis,
    retained_draw_list::tests::{command, DrawListWorld},
    retained_draw_list::UiNativeRetainedMutationCounters,
    UiNativeRetainedDrawList,
};
use worth_ui_host_contract::{
    UiMountedFrameIdentity, UiMountedLogicalDamage, UiMountedPaintCommandChange,
    UiMountedPaintOrderIntegrity, UiMountedPresentationDelta, UiMountedPresentationDeltaInput,
    UiMountedPresentationUnchanged, UiMountedPresentationUnchangedInput, UiMountedRgba8,
};

#[test]
fn cost_distinguishes_carried_normalized_selected_and_rasterized_damage() {
    let counters = UiNativeRetainedMutationCounters {
        draw_mutations: 2,
        order_mutations: 3,
        damage_rows_carried: 7,
        damage_regions: 2,
        damage_region_command_checks: 5,
        replayed_commands: 5,
        ..Default::default()
    };
    let cost = replay_cost([10, 10], counters, 4, 10, 20, 3, 11).unwrap();
    assert_eq!(cost.delta_rows_carried(), 23);
    assert_eq!(cost.logical_damage_regions(), 2);
    assert_eq!(cost.damage_region_command_checks(), 5);
    assert_eq!(cost.intersecting_commands(), 5);
    assert_eq!(cost.replayed_commands(), 3);
}

#[test]
fn offscreen_delta_advances_retained_truth_without_physical_work() {
    let world = DrawListWorld::new();
    let predecessor = UiMountedFrameIdentity::mint_unbound().unwrap();
    let old = world.rect(
        predecessor,
        world.first,
        200.0,
        UiMountedRgba8::new(1, 2, 3, 255),
    );
    let initial = world.initial(predecessor, [old]);
    let mut retained = UiNativeRetainedDrawList::initial(&initial, &[]).unwrap();
    let successor = UiMountedFrameIdentity::mint_unbound().unwrap();
    let replacement = world.rect(
        successor,
        world.first,
        220.0,
        UiMountedRgba8::new(4, 5, 6, 255),
    );
    let identity = command(replacement).identity();
    let delta = UiMountedPresentationDelta::from_inert_mechanics(UiMountedPresentationDeltaInput {
        predecessor,
        successor,
        surface: world.surface,
        binding: world.binding,
        content: world.content,
        baseline: world.requirement.baseline(),
        changes: vec![UiMountedPaintCommandChange::replacement(
            command(old).identity(),
            command(replacement),
        )],
        nodes: Vec::new(),
        order: Vec::new(),
        order_integrity: UiMountedPaintOrderIntegrity::for_order(&[
            worth_ui_host_contract::UiMountedPaintOrderIdentity::for_command(identity),
        ]),
        damage: [old.bounds(), replacement.bounds()]
            .map(UiMountedLogicalDamage::from_runtime_mounting)
            .to_vec(),
        auxiliary: None,
        production_cost: Default::default(),
    });

    let atlas = crate::native::text_atlas::UiNativeTextAtlas::new();
    retained
        .initialize_physical_coverage(UiNativeRasterBasis::new([100, 100], 1.0), &atlas)
        .unwrap();
    let (plan, _committed_undo, _effects) = prepare_delta_plan(
        UiNativeRasterBasis::new([100, 100], 1.0),
        &delta,
        &[],
        &atlas,
        &mut retained,
    )
    .unwrap_or_else(|_| panic!("a valid offscreen delta must plan without GPU effects"));
    assert!(plan.operations.is_empty());
    assert!(!plan.clear_retained_target);
    assert_eq!(plan.cost.presented_surfaces(), 0);
    assert_eq!(plan.cost.surface_acquisitions(), 0);
    assert_eq!(plan.cost.queue_submissions(), 0);
    assert_eq!(plan.cost.presents(), 0);
    assert_eq!(retained.command(identity), Some(&command(replacement)));

    let unchanged_frame = UiMountedFrameIdentity::mint_unbound().unwrap();
    retained
        .apply_unchanged(&UiMountedPresentationUnchanged::from_inert_mechanics(
            UiMountedPresentationUnchangedInput {
                predecessor: successor,
                successor: unchanged_frame,
                surface: world.surface,
                binding: world.binding,
                content: world.content,
                baseline: world.requirement.baseline(),
                production_cost: Default::default(),
            },
        ))
        .unwrap();
    let final_frame = UiMountedFrameIdentity::mint_unbound().unwrap();
    retained
        .apply_unchanged(&UiMountedPresentationUnchanged::from_inert_mechanics(
            UiMountedPresentationUnchangedInput {
                predecessor: unchanged_frame,
                successor: final_frame,
                surface: world.surface,
                binding: world.binding,
                content: world.content,
                baseline: world.requirement.baseline(),
                production_cost: Default::default(),
            },
        ))
        .unwrap();
}

#[test]
fn physical_delta_cost_exposes_the_full_surface_amplification_boundary() {
    let cost = replay_cost(
        [160, 96],
        UiNativeRetainedMutationCounters::default(),
        2,
        64,
        32,
        1,
        0,
    )
    .unwrap();
    assert_eq!(cost.presented_pixels(), 15_360);
    assert_eq!(cost.gpu_writes(), 1);
    assert_eq!(cost.render_passes(), 2);
    assert_eq!(cost.surface_copies(), 1);
    assert_eq!(cost.surface_acquisitions(), 1);
    assert_eq!(cost.queue_submissions(), 1);
    assert_eq!(cost.presents(), 1);
}

#[test]
fn fractional_damage_replays_unchanged_solids_sharing_only_physical_edge_pixels() {
    use crate::native::presentation::UiNativeRasterOperation;
    use worth_ui_host_contract::{
        UiMountedCanonicalBox, UiMountedCanonicalBoxInput, UiMountedCoordinateSpace,
        UiMountedPaintOrderIdentity,
    };

    let world = DrawListWorld::new();
    let predecessor = UiMountedFrameIdentity::mint_unbound().unwrap();
    let color = UiMountedRgba8::new(31, 67, 109, 128);
    let rect = world.rect(predecessor, world.first, 10.3, color);
    let initial = world.initial(predecessor, [rect]);
    let mut retained = UiNativeRetainedDrawList::initial(&initial, &[]).unwrap();
    let basis = UiNativeRasterBasis::new([100, 100], 1.25);
    let atlas = crate::native::text_atlas::UiNativeTextAtlas::new();
    retained
        .initialize_physical_coverage(basis, &atlas)
        .unwrap();
    let damage = [10.0, 42.35].map(|x| {
        UiMountedLogicalDamage::from_runtime_mounting(
            UiMountedCanonicalBox::canonicalize(UiMountedCanonicalBoxInput {
                x,
                y: 1.0,
                width: 0.01,
                height: 1.0,
                coordinate_space: UiMountedCoordinateSpace::HostSurface,
            })
            .unwrap(),
        )
    });
    // Neither logical region intersects the rectangle. Both clear a physical
    // edge pixel of it after 125% DPI snapping, so both need its paint replayed.
    assert!(damage[0].bounds().x() + damage[0].bounds().width() < rect.bounds().x());
    assert!(damage[1].bounds().x() > rect.bounds().x() + rect.bounds().width());
    let delta = UiMountedPresentationDelta::from_inert_mechanics(UiMountedPresentationDeltaInput {
        predecessor,
        successor: UiMountedFrameIdentity::mint_unbound().unwrap(),
        surface: world.surface,
        binding: world.binding,
        content: world.content,
        baseline: world.requirement.baseline(),
        changes: Vec::new(),
        nodes: Vec::new(),
        order: Vec::new(),
        order_integrity: UiMountedPaintOrderIntegrity::for_order(&[
            UiMountedPaintOrderIdentity::for_command(command(rect).identity()),
        ]),
        damage: damage.to_vec(),
        auxiliary: None,
        production_cost: Default::default(),
    });
    let (plan, _, _) = prepare_delta_plan(basis, &delta, &[], &atlas, &mut retained)
        .unwrap_or_else(|_| panic!("physical edge damage must plan"));
    let operations = plan
        .operations
        .iter()
        .map(|operation| match operation {
            UiNativeRasterOperation::Clear(rect) => (rect.physical_bounds(), [0; 4]),
            UiNativeRasterOperation::FilledRect { rect, source_rgba8 } => {
                (rect.physical_bounds(), *source_rgba8)
            }
            UiNativeRasterOperation::Glyph(_) => panic!("solid-only world"),
        })
        .collect::<Vec<_>>();
    assert_eq!(
        operations,
        vec![
            ([12.0, 1.0, 1.0, 2.0], [0; 4]),
            ([12.0, 1.0, 1.0, 2.0], color.channels()),
            ([52.0, 1.0, 1.0, 2.0], [0; 4]),
            ([52.0, 1.0, 1.0, 2.0], color.channels()),
        ]
    );
    assert_eq!(plan.cost.replayed_commands(), 2);
}
