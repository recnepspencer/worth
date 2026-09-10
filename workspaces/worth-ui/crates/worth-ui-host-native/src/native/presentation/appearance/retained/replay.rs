//! Select retained appearance commands for one exact damage region.
use super::*;

impl UiNativeAppearanceRetained {
    pub(crate) fn replay_for_damage(
        &mut self,
        damage: UiNativeAppearanceDamageRect,
    ) -> Result<Box<[UiNativeAppearanceCommandKey]>, UiNativeAppearanceRetainedDenial> {
        if damage.is_empty() {
            return Err(UiNativeAppearanceRetainedDenial::EmptyDamage);
        }
        let query = self
            .damage_index
            .intersecting(
                candidate_bounds(damage).map_err(UiNativeAppearanceRetainedDenial::Geometry)?,
            )
            .map_err(map_index_denial)?;
        self.counters.damage_queries += 1;
        self.counters.damage_branch_probes += query.branch_aabb_probes;
        self.counters.damage_leaf_probes += query.leaf_command_bounds_probes;
        let candidates = query
            .identities
            .into_iter()
            .filter(|key| {
                self.commands.get(key).is_some_and(|command| {
                    command.text_coverage().map_or_else(
                        || {
                            self.damage_bounds
                                .get(key)
                                .is_some_and(|bounds| bounds.intersects(damage))
                        },
                        |regions| regions.iter().any(|region| region.intersects(damage)),
                    )
                })
            })
            .collect::<Vec<_>>();
        let ordered = self.ordered_subset_keys(candidates)?;
        self.counters.replayed_commands += ordered.len();
        Ok(ordered)
    }
}
