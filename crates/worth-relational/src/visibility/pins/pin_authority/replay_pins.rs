//! Replay pin effects require custody of every selected resident mirror cell.
use super::VisibilityPinAuthority;
use crate::identity::data::VersionId;
use crate::runtime::{PartitionEdition, RelationalRuntime};
use crate::storage::overlay::{PartitionAccess, SnapshotPartitionPins};
use crate::storage::residency::{Full, RelationalResidencyLease};
use crate::storage::substrate::{EntityRecordKind, PinClass, RecordKind, RelationRecordKind};
use crate::visibility::snapshot_states::SnapshotState;

/// Move-only retention custody. Its constructor demands the mirror before
/// pinning, and release retains both its pin set and counted resident edition.
#[derive(Debug)]
pub(crate) struct ResidentReplayPins {
    state: SnapshotState,
    mirror: PartitionEdition,
}

impl ResidentReplayPins {
    pub(crate) fn state(&self) -> &SnapshotState {
        &self.state
    }

    fn visit_partitions(
        &self,
        mut visit: impl FnMut(&SnapshotPartitionPins, &RelationalResidencyLease<Full>),
    ) {
        for (id, pins) in &self.state.pinned_partitions {
            // A branch-only record can be absent from the legacy main mirror.
            // Present cells were demanded before this custody was constructed.
            if let Some(resident) = self.mirror.shared_partition(*id) {
                visit(pins, &resident);
            }
        }
    }
}

impl VisibilityPinAuthority<'_> {
    pub(crate) fn demand_replay_pins(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        state: SnapshotState,
    ) -> Result<ResidentReplayPins, crate::execution::RelationalExecutionDenialCause> {
        let mirror = self.runtime.acquire_partition_edition();
        for id in state.pinned_partitions.keys() {
            mirror
                .demand_full_partition(execution, *id)
                .map_err(crate::execution::RelationalExecutionDenialCause::from)?;
        }
        self.resident_replay_pins_in(state, mirror)
            .map_err(crate::execution::RelationalExecutionDenialCause::StorageUnavailable)
    }

    /// Resident-only admission refuses cold pinning without performing I/O.
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
        for (id, _) in mirror.cells() {
            if state.pinned_partitions.contains_key(&id) {
                mirror.resident_pin_lease(id)?;
            }
        }
        Ok(ResidentReplayPins { state, mirror })
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
