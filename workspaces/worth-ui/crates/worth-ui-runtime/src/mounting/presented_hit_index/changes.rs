use super::*;
use crate::mounting::spatial_index::UiMountedSpatialBudget;
use std::collections::{BTreeMap, BTreeSet};

/// Derived neighborhood evidence. The enclosing committed retention transition
/// supplies presentation authority; neither this set nor its indexes can issue it.
pub(crate) struct UiPresentedHitChanges {
    previous: UiPresentedHitIndex,
    current: UiPresentedHitIndex,
    changed: BTreeMap<UiSurfaceBindingGeneration, BTreeSet<UiMountedInstanceIdentity>>,
    comparison_steps: usize,
    comparison_key_probes: usize,
}

impl UiPresentedHitChanges {
    pub(in crate::mounting) fn between(
        previous: UiPresentedHitIndex,
        current: UiPresentedHitIndex,
    ) -> Self {
        let (keys, work) = current.rows.changed_keys_with_work(&previous.rows);
        let mut changed = BTreeMap::<_, BTreeSet<_>>::new();
        let mut comparison_key_probes = 0;
        for key in keys {
            let (old, old_probes) = previous.rows.get_with_probes(&key);
            let (new, new_probes) = current.rows.get_with_probes(&key);
            comparison_key_probes += old_probes + new_probes;
            let old = signature(old);
            let new = signature(new);
            if old != new {
                for signature in [old, new].into_iter().flatten() {
                    changed.entry(signature.0).or_default().insert(key);
                }
            }
        }
        Self {
            previous,
            current,
            changed,
            comparison_steps: work.cursor_steps(),
            comparison_key_probes,
        }
    }

    pub(crate) const fn comparison_steps(&self) -> usize {
        self.comparison_steps
    }

    pub(crate) const fn comparison_key_probes(&self) -> usize {
        self.comparison_key_probes
    }

    pub(crate) fn affects(
        &self,
        binding: UiSurfaceBindingGeneration,
        point: [f64; 2],
        previous_target: Option<UiMountedInstanceIdentity>,
    ) -> Result<(bool, UiHitTestSpatialWork), UiPresentedHitQueryDenial> {
        let mut work = UiHitTestSpatialWork::default();
        let Some(changed) = self.changed.get(&binding) else {
            return Ok((false, work));
        };
        if previous_target.is_some_and(|target| changed.contains(&target)) {
            return Ok((true, work));
        }
        let budget = UiMountedSpatialBudget {
            node_visits: 1024,
            candidates: 256,
        };
        for index in [&self.previous, &self.current] {
            let query = index
                .at_point(binding, point, budget)
                .map_err(|denial| match denial {
                    UiPresentedHitQueryDenial::NodeBudget { work: current } => {
                        work.merge(current);
                        UiPresentedHitQueryDenial::NodeBudget { work }
                    }
                    UiPresentedHitQueryDenial::CandidateBudget { work: current } => {
                        work.merge(current);
                        UiPresentedHitQueryDenial::CandidateBudget { work }
                    }
                    other => other,
                })?;
            work.merge(query.work);
            if query
                .rows
                .iter()
                .any(|row| changed.contains(&row.mounted_instance()))
            {
                return Ok((true, work));
            }
        }
        Ok((false, work))
    }

    #[cfg(test)]
    pub(crate) fn changed_count(&self) -> usize {
        self.changed.values().map(|members| members.len()).sum()
    }
}

fn signature(
    record: Option<&Record>,
) -> Option<(
    worth_ui_host_contract::UiSurfaceBindingGeneration,
    worth_ui_host_contract::UiMountedCanonicalBox,
    worth_ui_host_contract::UiMountedCanonicalBox,
    worth_ui_host_contract::UiMountedHitTestOrder,
)> {
    let row = record?.effective?;
    Some((
        row.mounted().binding(),
        row.bounds(),
        row.clip_bounds(),
        row.order(),
    ))
}
