use super::*;
use crate::native::presentation::retained_draw_list::UiNativeRetainedMutationCounters;

#[test]
fn one_tail_surface_damage_uses_bounded_semantic_and_rank_lookups_at_512_commands() {
    const COMMANDS: usize = 512;
    let world = DrawListWorld::new();
    let frame = worth_ui_host_contract::UiMountedFrameIdentity::mint_unbound().unwrap();
    let rows = (0..COMMANDS)
        .map(|index| {
            world.rect_at_order(
                frame,
                UiMountedInstanceIdentity::mint_unbound().unwrap(),
                (index * 40) as f32,
                UiMountedRgba8::new(20, 30, 40, 255),
                index as u32,
            )
        })
        .collect::<Vec<_>>();
    let commands = rows.iter().copied().map(command).collect::<Vec<_>>();
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
    let tail = *rows.last().unwrap();
    let mut appearance =
        UiNativeAppearanceRetained::new(UiNativeAppearanceScale::qualified(1_000).unwrap());
    appearance
        .insert(
            UiNativeAppearanceCommand::Surface(surface_at(tail, (COMMANDS - 1) as u32)),
            None,
        )
        .unwrap();
    retained.staged_appearance = Some((world.requirement, appearance));

    retained.order.take_cost();
    let extent = [21_000, 100];
    let left = (COMMANDS as u32 - 1) * 40;
    let clear = crate::native::presentation::raster::raster_physical_bounds(
        [left, 0, left + 32, 24],
        extent,
    )
    .unwrap();
    let mut counters = UiNativeRetainedMutationCounters::default();
    let operations = retained
        .appearance_operations_for_damage(
            clear,
            crate::native::presentation::raster::UiNativeRasterBasis::new(extent, 1.0),
            &[command(tail).identity()],
            &crate::native::text_atlas::UiNativeTextAtlas::new(),
            &mut counters,
        )
        .unwrap();

    assert_eq!(operations.len(), 1);
    assert_eq!(counters.order_index_lookups, 2);
    assert!(counters.order_index_node_touches <= 32);
    assert_eq!(counters.retained_command_scans, 0);
}
