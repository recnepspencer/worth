use std::collections::BTreeMap;

use crate::identity::data::VersionId;

#[derive(Debug, Default)]
pub(crate) struct ReplayRetentionIndex {
    pub(crate) entries: BTreeMap<VersionId, ReplayRetentionState>,
}

impl ReplayRetentionIndex {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn fork(&self) -> Self {
        Self::new()
    }

    pub(crate) fn retained_mut(
        &mut self,
        version_id: VersionId,
    ) -> Option<&mut ReplayRetentionState> {
        self.entries.get_mut(&version_id)
    }

    pub(crate) fn insert_retained(&mut self, version_id: VersionId, state: ReplayRetentionState) {
        self.entries.insert(version_id, state);
    }

    pub(crate) fn take_retained(&mut self, version_id: VersionId) -> Option<ReplayRetentionState> {
        self.entries.remove(&version_id)
    }

    pub(crate) fn versions(&self) -> impl Iterator<Item = VersionId> + '_ {
        self.entries.keys().copied()
    }
}

#[derive(Debug)]
pub(crate) struct ReplayRetentionState {
    pub(crate) ref_count: usize,
    pub(crate) pins: crate::visibility::pins::ResidentReplayPins,
}

impl super::VisibilitySubsystem {
    /// Cold admission precedes this transition. Serialize only resident pin
    /// effects and reference ownership, including the slow-path recheck.
    pub(crate) fn retain_resident_replay(
        &self,
        version_id: VersionId,
        pins: crate::visibility::pins::ResidentReplayPins,
        pin: impl FnOnce(&crate::visibility::pins::ResidentReplayPins),
    ) -> bool {
        let mut retention = self.replay_retention.write();
        if let Some(retained) = retention.retained_mut(version_id) {
            retained.ref_count += 1;
            return false;
        }
        pin(&pins);
        retention.insert_retained(version_id, ReplayRetentionState { ref_count: 1, pins });
        true
    }

    pub(crate) fn release_resident_replay(
        &self,
        version_id: VersionId,
        unpin: impl FnOnce(&crate::visibility::pins::ResidentReplayPins),
    ) -> bool {
        let mut retention = self.replay_retention.write();
        let Some(retained) = retention.retained_mut(version_id) else {
            return false;
        };
        if retained.ref_count > 1 {
            retained.ref_count -= 1;
        } else {
            // Release before removing the entry so a new first retain cannot
            // overlap physical effects for the previous reference set.
            unpin(&retained.pins);
            retention.take_retained(version_id);
        }
        true
    }
}
