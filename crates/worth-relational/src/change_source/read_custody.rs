//! Actual owners retaining reads at a refused Native close cut.
use crate::history::retention::RelationalRetentionCostCounters;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum RelationalReadCustodyHolders {
    Registrations {
        active_snapshot_handles: bool,
        published_snapshot_handles: bool,
        published_snapshot_capacity: bool,
        replay_versions: bool,
        fork_targets: bool,
    },
    SnapshotsAndHistory {
        active_snapshots: usize,
        detached_history: bool,
    },
    DetachedHistory {
        observations: u64,
        transactions: u64,
        candidates: u64,
        performed_settlements: u64,
        external_pins: u64,
    },
    BranchHead {
        index: usize,
    },
    CheckpointDirectory {
        index: usize,
        canonical: bool,
        shares: usize,
    },
}
impl RelationalReadCustodyHolders {
    pub(crate) fn detached(c: RelationalRetentionCostCounters) -> Self {
        Self::DetachedHistory {
            observations: c
                .observation_acquires
                .saturating_sub(c.observation_releases),
            transactions: c
                .transaction_acquires
                .saturating_sub(c.transaction_releases),
            candidates: c.candidate_acquires.saturating_sub(c.candidate_releases),
            performed_settlements: c
                .performed_settlement_acquires
                .saturating_sub(c.performed_settlement_releases),
            external_pins: c
                .external_pin_acquires
                .saturating_sub(c.external_pin_releases),
        }
    }
}
