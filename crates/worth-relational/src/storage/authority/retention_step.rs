//! One storage/allocator handoff shared by explicit and legacy retention passes.
use super::StorageAuthority;
use crate::execution::RelationalExecutionDenialCause as Cause;
use crate::history::data::RecordAllocationClass;
use crate::identity::data::{PartitionId, VersionId};
use crate::runtime::TryPartitionMutationDenied;
use crate::storage::data::RecordLifecycleState as State;
use crate::storage::substrate::RecordKind;

/// Why an explicit pass stopped; its completed progress is retained separately.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RelationalRetentionPassStop {
    Request(Cause),
    /// A live authoritative map edition prevents this explicit try-pass from
    /// writing. Historical branch roots do not themselves imply this refusal.
    StorageObserved,
}

#[derive(Default)]
pub(crate) struct RetentionStep {
    pub(crate) reclaimable: bool,
    pub(crate) refreshed: bool,
    pub(crate) reclaimed: bool,
}

type Preparation<'a> = Option<&'a mut dyn FnMut(u64, u64) -> Result<(), Cause>>;

impl StorageAuthority<'_> {
    pub(crate) fn retention_step<K: RecordKind>(
        &self,
        class: RecordAllocationClass,
        partition_id: PartitionId,
        slot: usize,
        fence: VersionId,
        reclaim: bool,
        mut prepare: Preparation<'_>,
    ) -> Result<RetentionStep, RelationalRetentionPassStop> {
        use RelationalRetentionPassStop as Stop;
        let mut writer = self.runtime.edit_partitions();
        let partition = if let Some(admit) = prepare.as_mut() {
            writer
                .try_partition_mut(partition_id, &mut |work, bytes| admit(work, bytes))
                .map_err(|stop| match stop {
                    TryPartitionMutationDenied::Observed => Stop::StorageObserved,
                    TryPartitionMutationDenied::SizeOverflow => {
                        Stop::Request(Cause::ChargedBytesOverflow)
                    }
                    TryPartitionMutationDenied::Admission(cause) => Stop::Request(cause),
                })?
        } else {
            writer.partition_mut(partition_id)
        };
        let Some(partition) = partition else {
            return Ok(RetentionStep::default());
        };
        let arena = K::arena_mut(partition);
        if let Some(admit) = prepare.as_mut() {
            admit(
                arena
                    .retention_inspection_work_bound()
                    .ok_or(Stop::Request(Cause::WorkCounterOverflow))?,
                0,
            )
            .map_err(Stop::Request)?;
        }
        let Some(retired) = arena.retired_at_for_slot(slot) else {
            return Ok(RetentionStep::default());
        };
        let previous = arena.get_slot(slot).map(|old| old.lifecycle());
        let lifecycle = self.retention_lifecycle::<K>(arena, slot, Some(retired), fence);
        let reclaimable = lifecycle == State::Reclaimable;
        if reclaimable && reclaim {
            let reclaimed = crate::runtime::ReclaimedRecordSlot::new(class, partition_id, slot);
            if let Some(admit) = prepare.as_mut() {
                admit(
                    arena
                        .retention_write_work_bound(true)
                        .ok_or(Stop::Request(Cause::WorkCounterOverflow))?,
                    0,
                )
                .map_err(Stop::Request)?;
                let bytes = arena
                    .retention_reset_bound(slot)
                    .ok_or(Stop::Request(Cause::ChargedBytesOverflow))?;
                self.runtime
                    .record_identity
                    .reclaim_after_admission(
                        reclaimed,
                        bytes,
                        &mut |work, bytes| admit(work, bytes),
                        || {
                            arena.set_lifecycle_for_slot(slot, State::Reusable);
                            arena.reset_slot(slot);
                        },
                    )
                    .map_err(Stop::Request)?;
            } else {
                arena.set_lifecycle_for_slot(slot, State::Reusable);
                arena.reset_slot(slot);
                self.runtime.record_identity.admit_reclaimed(reclaimed);
            }
            return Ok(RetentionStep {
                reclaimable,
                refreshed: false,
                reclaimed: true,
            });
        }
        let refreshed = previous.is_some_and(|old| old != lifecycle);
        if refreshed {
            if let Some(admit) = prepare.as_mut() {
                admit(
                    arena
                        .retention_write_work_bound(false)
                        .ok_or(Stop::Request(Cause::WorkCounterOverflow))?,
                    arena
                        .retention_refresh_bound(slot)
                        .ok_or(Stop::Request(Cause::ChargedBytesOverflow))?,
                )
                .map_err(Stop::Request)?;
            }
            arena.set_lifecycle_for_slot(slot, lifecycle);
        }
        Ok(RetentionStep {
            reclaimable,
            refreshed,
            reclaimed: false,
        })
    }
}
