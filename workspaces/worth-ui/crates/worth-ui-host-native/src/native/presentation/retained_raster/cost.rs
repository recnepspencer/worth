pub(in crate::native::presentation) fn replay_cost(
    extent: [u32; 2],
    counters: super::super::retained_draw_list::UiNativeRetainedMutationCounters,
    operation_count: usize,
    cleared_pixels: u64,
    rendered_pixels: u64,
    replayed_commands: u64,
    node_changes: usize,
) -> Result<
    worth_ui_host_contract::UiHostPresentationCostReport,
    worth_ui_host_contract::UiHostSurfacePresentationDenial,
> {
    let physical = operation_count > 0;
    let operations = u64::try_from(operation_count).map_err(|_| super::malformed())?;
    let node_changes = u64::try_from(node_changes).map_err(|_| super::malformed())?;
    let damage_index_probes = counters
        .damage_index_branch_aabb_probes
        .checked_add(counters.damage_index_leaf_command_bounds_probes)
        .ok_or_else(super::malformed)?;
    let presented_pixels = u64::from(extent[0]) * u64::from(extent[1]);
    Ok(
        worth_ui_host_contract::UiHostPresentationCostReport::from_adapter(
            worth_ui_host_contract::UiHostPresentationCostInput {
                presented_surfaces: u64::from(physical),
                translated_rows: counters
                    .draw_mutations
                    .checked_add(node_changes)
                    .ok_or_else(super::malformed)?,
                native_resource_cache_hits: counters.replayed_commands,
                delta_rows_carried: counters
                    .draw_mutations
                    .checked_add(node_changes)
                    .and_then(|value| value.checked_add(counters.order_mutations))
                    .and_then(|value| value.checked_add(counters.damage_rows_carried))
                    .ok_or_else(super::malformed)?,
                draw_list_mutations: counters.draw_mutations,
                order_mutations: counters.order_mutations,
                order_index_lookups: counters.order_index_lookups,
                order_index_node_touches: counters.order_index_node_touches,
                order_index_rotations: counters.order_index_rotations,
                order_index_high_water: counters.order_index_high_water,
                logical_damage_regions: counters.damage_regions,
                damage_index_probes,
                damage_index_stored_records: counters.damage_index_stored_records,
                damage_index_high_water: counters.damage_index_high_water,
                damage_region_command_checks: counters.damage_region_command_checks,
                intersecting_commands: counters.replayed_commands,
                replayed_commands,
                cleared_pixels,
                rendered_pixels,
                presented_pixels: if physical { presented_pixels } else { 0 },
                gpu_writes: u64::from(physical && operations > 0),
                render_passes: if physical { 2 } else { 0 },
                surface_copies: u64::from(physical),
                surface_acquisitions: u64::from(physical),
                queue_submissions: u64::from(physical),
                presents: u64::from(physical),
                ..Default::default()
            },
        ),
    )
}
