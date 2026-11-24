//! Observe an admitted basis while its existing external or lane gate is held.
use super::{lock_lane, RelationalBranchRetentionLaneState, RelationalRetentionCostCounters};
use crate::branch::basis::AdmittedRelationalBranchBasis;

/// Which existing retention gate a trial holds.
#[doc(hidden)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelationalRetentionGateForTest {
    /// The retention owner's external capture cut.
    ExternalCut,
    /// The binding lane's state lock.
    Lane,
}

/// One lane's counters and obligation census.
#[doc(hidden)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RelationalRetentionCensusForTest {
    pub counters: RelationalRetentionCostCounters,
    pub lane_obligations: usize,
    /// Obligations this lane holds on the census root, if any.
    pub root_obligations: Option<usize>,
}

fn census(
    lane: &RelationalBranchRetentionLaneState,
    root_id: u64,
) -> RelationalRetentionCensusForTest {
    RelationalRetentionCensusForTest {
        counters: lane.counters,
        lane_obligations: lane.obligations.len(),
        root_obligations: lane.root_obligations.get(&root_id).copied(),
    }
}

impl AdmittedRelationalBranchBasis {
    #[doc(hidden)]
    pub fn retention_census_for_test(&self) -> RelationalRetentionCensusForTest {
        let lane = lock_lane(&self.inner.retention_binding.lane);
        census(&lane, self.inner.root.id())
    }

    /// Run `step` once with `gate` held. Returns the census read before the
    /// gate was taken (external cut) or right after it (lane lock), then again
    /// while it is still held, or `None` without running `step` when the
    /// retention owner is gone.
    #[doc(hidden)]
    pub fn with_gate_held_for_test<R>(
        &self,
        gate: RelationalRetentionGateForTest,
        step: impl FnOnce() -> R,
    ) -> Option<(
        R,
        RelationalRetentionCensusForTest,
        RelationalRetentionCensusForTest,
    )> {
        let binding = &self.inner.retention_binding;
        let root_id = self.inner.root.id();
        let owner = binding.owner.upgrade()?;
        Some(match gate {
            RelationalRetentionGateForTest::ExternalCut => {
                let before = self.retention_census_for_test();
                let cut = owner
                    .external_cut
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner());
                let result = step();
                let after = self.retention_census_for_test();
                drop(cut);
                (result, before, after)
            }
            RelationalRetentionGateForTest::Lane => {
                let lane = lock_lane(&binding.lane);
                let before = census(&lane, root_id);
                let result = step();
                let after = census(&lane, root_id);
                drop(lane);
                (result, before, after)
            }
        })
    }
}
