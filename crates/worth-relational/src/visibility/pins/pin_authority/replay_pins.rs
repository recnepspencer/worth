use super::*;
use crate::identity::data::PartitionId;
use crate::storage::data::{
    RelationalStorageSubject, RelationalStorageUnavailable, RelationalStorageUnavailableCause,
};
use crate::storage::residency::{Full, RelationalResidencyLease};
use std::collections::BTreeMap;

impl VisibilityPinAuthority<'_> {
    pub(crate) fn pin_replay_state(
        &self,
        state: &SnapshotState,
    ) -> Result<BTreeMap<PartitionId, RelationalResidencyLease<Full>>, RelationalStorageUnavailable>
    {
        // Acquire the complete resident proof before changing any counter.
        let edition = self.runtime.partitions.acquire();
        let mut leases = BTreeMap::new();
        for id in state.pinned_partitions.keys() {
            leases.insert(*id, edition.resident_pin_lease(*id)?);
        }
        self.adjust_replay_state(state, &leases, 1)?;
        Ok(leases)
    }

    pub(crate) fn unpin_replay_state(
        &self,
        state: &SnapshotState,
        leases: &BTreeMap<PartitionId, RelationalResidencyLease<Full>>,
    ) -> Result<(), RelationalStorageUnavailable> {
        self.adjust_replay_state(state, leases, -1)
    }

    fn adjust_replay_state(
        &self,
        state: &SnapshotState,
        leases: &BTreeMap<PartitionId, RelationalResidencyLease<Full>>,
        delta: i32,
    ) -> Result<(), RelationalStorageUnavailable> {
        for (id, pins) in &state.pinned_partitions {
            let lease = leases.get(id).ok_or(RelationalStorageUnavailable {
                subject: RelationalStorageSubject::Region(*id),
                cause: RelationalStorageUnavailableCause::SourceUnavailable,
            })?;
            for slot in pins.entity_slots.iter_set_slots() {
                adjust_record_pin::<EntityRecordKind>(
                    self.runtime,
                    lease,
                    crate::identity::data::EntityId::new(*id, slot as u64, 0),
                    SubstratePinClass::Replay,
                    delta,
                )?;
            }
            for slot in pins.relation_slots.iter_set_slots() {
                adjust_record_pin::<RelationRecordKind>(
                    self.runtime,
                    lease,
                    crate::identity::data::RelationId::new(*id, slot as u64, 0),
                    SubstratePinClass::Replay,
                    delta,
                )?;
            }
        }
        Ok(())
    }
}
