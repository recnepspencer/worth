//! Hold one existing retention gate of a lane binding around one caller step.
//!
//! The gates are the owner's external cut and the lane's own state lock; this
//! module adds no gate, state or ordering. A trial reads the lane census before
//! the step and again while the same gate is still held, exactly where the
//! original in-crate fixture read them.
use super::{lock_lane, RelationalBranchRetentionBinding, RelationalRetentionCostCounters};

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

impl RelationalBranchRetentionBinding {
    pub(crate) fn census_for_test(&self, root_id: u64) -> RelationalRetentionCensusForTest {
        let lane = lock_lane(&self.lane);
        RelationalRetentionCensusForTest {
            counters: lane.counters,
            lane_obligations: lane.obligations.len(),
            root_obligations: lane.root_obligations.get(&root_id).copied(),
        }
    }

    /// Run `step` once with `gate` held. Returns the census read before the
    /// gate was taken (external cut) or right after it (lane lock), then again
    /// while it is still held, or `None` without running `step` when the
    /// retention owner is gone.
    pub(crate) fn with_gate_held_for_test<R>(
        &self,
        gate: RelationalRetentionGateForTest,
        root_id: u64,
        step: impl FnOnce() -> R,
    ) -> Option<(
        R,
        RelationalRetentionCensusForTest,
        RelationalRetentionCensusForTest,
    )> {
        let owner = self.owner.upgrade()?;
        Some(match gate {
            RelationalRetentionGateForTest::ExternalCut => {
                let before = self.census_for_test(root_id);
                let cut = owner
                    .external_cut
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner());
                let result = step();
                let after = self.census_for_test(root_id);
                drop(cut);
                (result, before, after)
            }
            RelationalRetentionGateForTest::Lane => {
                let lane = lock_lane(&self.lane);
                let census = |lane: &super::RelationalBranchRetentionLaneState| {
                    RelationalRetentionCensusForTest {
                        counters: lane.counters,
                        lane_obligations: lane.obligations.len(),
                        root_obligations: lane.root_obligations.get(&root_id).copied(),
                    }
                };
                let before = census(&lane);
                let result = step();
                let after = census(&lane);
                drop(lane);
                (result, before, after)
            }
        })
    }
}
