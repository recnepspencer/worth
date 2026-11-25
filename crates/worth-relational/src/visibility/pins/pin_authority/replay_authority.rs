//! Replay pin effects require custody of every selected resident mirror cell.
use super::VisibilityPinAuthority;
use crate::identity::data::VersionId;
use crate::runtime::state::subsystems::storage::partition_edition::PartitionEdition;
use crate::runtime::RelationalRuntime;
use crate::storage::residency::{Full, RelationalResidencyLease};
use crate::storage::substrate::{EntityRecordKind, PinClass, RecordKind, RelationRecordKind};
use crate::visibility::snapshot_states::SnapshotState;

use super::replay_pins::ResidentReplayPins;

impl VisibilityPinAuthority<'_> {
    pub(crate) fn demand_replay_pins(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        state: SnapshotState,
    ) -> Result<ResidentReplayPins, crate::execution::denial_cause::RelationalExecutionDenialCause>
    {
        let mirror = self.runtime.acquire_partition_edition();
        for id in state.pinned_partitions.keys() {
            mirror
                .demand_partition(execution, *id)
                .map_err(crate::execution::denial_cause::RelationalExecutionDenialCause::from)?;
        }
        self.resident_replay_pins_in(state, mirror).map_err(
            crate::execution::denial_cause::RelationalExecutionDenialCause::StorageUnavailable,
        )
    }

    /// Resident-only admission refuses cold pinning without performing I/O.
    #[cfg(test)]
    pub(crate) fn resident_replay_pins(
        &self,
        state: SnapshotState,
    ) -> Result<ResidentReplayPins, crate::storage::data::RelationalStorageUnavailable> {
        self.resident_replay_pins_in(state, self.runtime.acquire_partition_edition())
    }

    fn resident_replay_pins_in(
        &self,
        state: SnapshotState,
        mirror: PartitionEdition,
    ) -> Result<ResidentReplayPins, crate::storage::data::RelationalStorageUnavailable> {
        let mut leases = std::collections::BTreeMap::new();
        for (id, _) in mirror.cells() {
            if state.pinned_partitions.contains_key(&id) {
                leases.insert(id, mirror.resident_pin_lease(id)?);
            }
        }
        Ok(ResidentReplayPins {
            state,
            mirror: leases,
        })
    }

    pub(crate) fn pin_replay_state(&self, pins: &ResidentReplayPins, retention_fence: VersionId) {
        self.adjust_replay_state(pins, 1, retention_fence);
    }

    pub(crate) fn unpin_replay_state(&self, pins: &ResidentReplayPins, retention_fence: VersionId) {
        self.adjust_replay_state(pins, -1, retention_fence);
    }

    fn adjust_replay_state(
        &self,
        pins: &ResidentReplayPins,
        delta: i32,
        retention_fence: VersionId,
    ) {
        pins.visit_partitions(|pins, resident| {
            for slot in pins.entity_slots.iter_set_slots() {
                adjust_record_pin::<EntityRecordKind>(
                    self.runtime,
                    resident,
                    slot,
                    delta,
                    retention_fence,
                );
            }
            for slot in pins.relation_slots.iter_set_slots() {
                adjust_record_pin::<RelationRecordKind>(
                    self.runtime,
                    resident,
                    slot,
                    delta,
                    retention_fence,
                );
            }
        });
    }
}

fn adjust_record_pin<K: RecordKind>(
    runtime: &RelationalRuntime,
    resident: &RelationalResidencyLease<Full>,
    slot: usize,
    delta: i32,
    retention_fence: VersionId,
) {
    runtime.storage_authority().adjust_named_pin::<K>(
        resident,
        slot,
        PinClass::Replay,
        delta,
        retention_fence,
    );
}
