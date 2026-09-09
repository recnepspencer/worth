use worth_ui_host_contract::{UiMountedLogicalDamage, UiMountedPaintOrderIdentity};

use super::{
    UiNativeRetainedDrawList, UiNativeRetainedDrawListDenial, UiNativeRetainedMutationCounters,
    UiNativeRetainedReplayPlan, UiNativeRetainedReplayRegion,
};
use crate::native::presentation::damage_regions::normalize_damage;

impl UiNativeRetainedDrawList {
    pub(super) fn replay_plan(
        &self,
        regions: &[UiMountedLogicalDamage],
        draw_mutations: usize,
        order_mutations: usize,
    ) -> Result<UiNativeRetainedReplayPlan, UiNativeRetainedDrawListDenial> {
        let mut counters = UiNativeRetainedMutationCounters {
            draw_mutations: exact_u64(draw_mutations)?,
            order_mutations: exact_u64(order_mutations)?,
            damage_rows_carried: exact_u64(regions.len())?,
            ..Default::default()
        };
        let clear_regions = normalize_damage(regions)?;
        counters.damage_regions = exact_u64(clear_regions.len())?;
        let mut replay_regions = Vec::with_capacity(clear_regions.len());
        for region in clear_regions {
            // Once admitted images are retained, selection waits for physical
            // damage conversion in the raster planner. Allocation bounds no
            // longer select an ordinary replay set at this stage.
            if self.physical_coverage.is_some() {
                replay_regions.push(UiNativeRetainedReplayRegion {
                    damage: region,
                    replay: Box::new([]),
                });
                continue;
            }
            let query = self.damage.intersecting(region.bounds())?;
            counters.damage_index_branch_aabb_probes = add(
                counters.damage_index_branch_aabb_probes,
                query.branch_aabb_probes,
            )?;
            counters.damage_index_leaf_command_bounds_probes = add(
                counters.damage_index_leaf_command_bounds_probes,
                query.leaf_command_bounds_probes,
            )?;
            counters.damage_index_stored_records = exact_u64(query.stored_records)?;
            counters.damage_index_high_water = exact_u64(query.high_water_records)?;
            let replay = self.order.ordered_subset(
                query
                    .identities
                    .into_iter()
                    .map(UiMountedPaintOrderIdentity::for_command),
            )?;
            counters.damage_region_command_checks =
                add(counters.damage_region_command_checks, replay.len())?;
            counters.replayed_commands = add(counters.replayed_commands, replay.len())?;
            replay_regions.push(UiNativeRetainedReplayRegion {
                damage: region,
                replay: replay
                    .into_iter()
                    .map(UiMountedPaintOrderIdentity::command)
                    .collect(),
            });
        }
        let order_cost = self.order.take_cost();
        counters.order_index_lookups = order_cost.identity_lookups();
        counters.order_index_node_touches = order_cost.node_touches();
        counters.order_index_rotations = order_cost.rotations();
        counters.order_index_high_water = order_cost.high_water_entries();
        Ok(UiNativeRetainedReplayPlan {
            physical_text_regions: Vec::new(),
            baseline_rgba8: self.baseline.transparent_rgba8(),
            regions: replay_regions.into_boxed_slice(),
            staged_appearance_regions: Box::new([]),
            counters,
            identity_overlay_effect: false,
        })
    }
}

fn exact_u64(value: usize) -> Result<u64, UiNativeRetainedDrawListDenial> {
    u64::try_from(value).map_err(|_| UiNativeRetainedDrawListDenial::CounterOverflow)
}

fn add(total: u64, value: usize) -> Result<u64, UiNativeRetainedDrawListDenial> {
    total
        .checked_add(exact_u64(value)?)
        .ok_or(UiNativeRetainedDrawListDenial::CounterOverflow)
}
