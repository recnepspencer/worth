//! The generic maintained contract selects journal records, never arena history.
use super::{membership::key, *};
use crate::storage::overlay::{PartitionMutationJournal, PartitionState};
use crate::storage::substrate::maintenance::{
    ChangeWork, CommitChange, Maintained, MaintenanceConsistency, MaintenanceRefusal,
    MaintenanceRoute, ReconstructionLane, ReconstructionWork, RecordChangeState,
};
use crate::storage::substrate::{RecordArena, RecordKind};
use crate::transactions::data::RecordRef;
use std::collections::BTreeSet;

#[derive(Debug)]
pub(crate) enum DirectoryError {
    Admission(MaintenanceRefusal),
    Corrupt,
}
impl From<MaintenanceRefusal> for DirectoryError {
    fn from(cause: MaintenanceRefusal) -> Self {
        Self::Admission(cause)
    }
}

pub(super) struct DirectoryMaintainer<'a> {
    pub(super) partition: &'a PartitionState,
    pub(super) before: &'a PartitionState,
    pub(super) routes: BTreeSet<MaintenanceRoute>,
}
impl Maintained for DirectoryMaintainer<'_> {
    type State = RegionRecordDirectory;
    type Error = DirectoryError;
    type Retirement = std::convert::Infallible;
    fn routes(&self) -> &BTreeSet<MaintenanceRoute> {
        &self.routes
    }
    fn consistency(&self) -> MaintenanceConsistency {
        MaintenanceConsistency::Synchronous
    }
    fn apply(
        &mut self,
        previous: &Arc<Self::State>,
        change: &CommitChange,
        work: &mut ChangeWork<'_>,
    ) -> Result<Arc<Self::State>, Self::Error> {
        let Some(changed) = change.partition(self.partition.partition_id) else {
            return Ok(previous.clone());
        };
        let mut next = previous.as_ref().clone();
        let mut any = false;
        for record in changed.records() {
            // Membership is the actual selected arena's live bit. In
            // particular audit-retained relations can remain current while
            // their lifecycle/retirement metadata is no longer Live/None.
            let before = record
                .before
                .as_ref()
                .filter(|state| selected(self.before, state));
            let after = record
                .after
                .as_ref()
                .filter(|state| selected(self.partition, state));
            if before.map(|state| (&state.record, state.kind))
                == after.map(|state| (&state.record, state.kind))
            {
                continue;
            }
            any |= update_record(&mut next, before, false, work)?;
            any |= update_record(&mut next, after, true, work)?;
        }
        Ok(if any {
            Arc::new(next)
        } else {
            previous.clone()
        })
    }
    fn cold(
        &mut self,
        lane: &ReconstructionLane<'_>,
        work: &mut ReconstructionWork<'_, '_>,
    ) -> Result<Arc<Self::State>, Self::Error> {
        RegionRecordDirectory::capture(lane, self.partition, &mut |units, bytes| {
            work.prepare(units, bytes).map_err(Into::into)
        })
    }
    fn retire(
        &mut self,
        _: &Arc<Self::State>,
        horizon: &Self::Retirement,
        _: &mut ChangeWork<'_>,
    ) -> Result<Arc<Self::State>, Self::Error> {
        match *horizon {}
    }
}
impl RegionRecordDirectory {
    pub(crate) fn updated(
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        previous: &Arc<Self>,
        before: &PartitionState,
        partition: &PartitionState,
        journal: &PartitionMutationJournal,
        change: Option<&CommitChange>,
    ) -> Result<Arc<Self>, DirectoryError> {
        if previous.partition != partition.partition_id {
            return Err(DirectoryError::Corrupt);
        }
        let mut owner = DirectoryMaintainer {
            partition,
            before,
            routes: BTreeSet::from([MaintenanceRoute::Partition(partition.partition_id)]),
        };
        ChangeWork::run_phase(execution, "record_directory", |work| {
            if let Some(change) = change {
                return crate::storage::substrate::maintenance::apply_change(
                    &mut owner,
                    Some(previous),
                    change,
                    work,
                );
            }
            owner.apply_journal(previous, journal, work)
        })
    }
}
impl DirectoryMaintainer<'_> {
    fn apply_journal(
        &mut self,
        previous: &Arc<RegionRecordDirectory>,
        journal: &PartitionMutationJournal,
        work: &mut ChangeWork<'_>,
    ) -> Result<Arc<RegionRecordDirectory>, DirectoryError> {
        let mut next = previous.as_ref().clone();
        let mut any = false;
        for &slot in &journal.entity_slots {
            let old = live_state(&self.before.entity_arena, slot, RecordRef::Entity);
            let current = live_state(&self.partition.entity_arena, slot, RecordRef::Entity);
            if old == current {
                continue;
            }
            any |= update_record(&mut next, old.as_ref(), false, work)?;
            any |= update_record(&mut next, current.as_ref(), true, work)?;
        }
        for &slot in &journal.relation_slots {
            let old = live_state(&self.before.relation_arena, slot, RecordRef::Relation);
            let current = live_state(&self.partition.relation_arena, slot, RecordRef::Relation);
            if old == current {
                continue;
            }
            any |= update_record(&mut next, old.as_ref(), false, work)?;
            any |= update_record(&mut next, current.as_ref(), true, work)?;
        }
        Ok(if any {
            Arc::new(next)
        } else {
            previous.clone()
        })
    }
}
fn update_record(
    directory: &mut RegionRecordDirectory,
    state: Option<&RecordChangeState>,
    present: bool,
    work: &mut ChangeWork<'_>,
) -> Result<bool, DirectoryError> {
    let Some(state) = state else {
        return Ok(false);
    };
    match state.record {
        RecordRef::Entity(id) => {
            directory
                .entities
                .update(key(state.kind, id), present, &mut |units, bytes| {
                    work.prepare(units, bytes).map_err(Into::into)
                })?
        }
        RecordRef::Relation(id) => {
            directory
                .relations
                .update(key(state.kind, id), present, &mut |units, bytes| {
                    work.prepare(units, bytes).map_err(Into::into)
                })?
        }
    }
    Ok(true)
}
fn live_state<K: RecordKind>(
    arena: &RecordArena<K>,
    slot: usize,
    wrap: impl FnOnce(crate::identity::data::RecordId<K::Domain>) -> RecordRef,
) -> Option<RecordChangeState> {
    if arena.live_bitset.count_ones_in_range(slot, slot + 1) == 0 {
        return None;
    }
    let record = arena.get_slot(slot)?;
    Some(RecordChangeState {
        record: wrap(crate::identity::data::RecordId::new(
            record.partition_id(),
            slot as u64,
            record.generation(),
        )),
        kind: record.kind_id()?,
        lifecycle: record.lifecycle(),
        retired_at: record.retired_at(),
    })
}

impl DirectoryError {
    pub(crate) fn into_failure(
        self,
        partition: PartitionId,
    ) -> crate::storage::residency::RelationalResidencyDemandFailure {
        match self {
            Self::Admission(MaintenanceRefusal::Execution(cause)) => {
                crate::storage::residency::RelationalResidencyDemandFailure::Request(cause)
            }
            _ => crate::storage::data::RelationalStorageUnavailable {
                subject: crate::storage::data::RelationalStorageSubject::Region(partition),
                cause: crate::storage::data::RelationalStorageUnavailableCause::IntegrityMismatch,
            }
            .into(),
        }
    }
    pub(crate) fn into_content_failure(
        self,
        partition: PartitionId,
    ) -> crate::storage::residency::RegionContentFailure {
        match self {
            Self::Admission(MaintenanceRefusal::Execution(cause)) => {
                crate::storage::residency::RegionContentFailure::Execution(cause)
            }
            _ => crate::storage::residency::RegionContentFailure::Storage(
                crate::storage::data::RelationalStorageUnavailable {
                    subject: crate::storage::data::RelationalStorageSubject::Region(partition),
                    cause:
                        crate::storage::data::RelationalStorageUnavailableCause::IntegrityMismatch,
                },
            ),
        }
    }
}

fn selected(partition: &PartitionState, state: &RecordChangeState) -> bool {
    match state.record {
        RecordRef::Entity(id) => {
            partition
                .entity_arena
                .live_bitset
                .count_ones_in_range(id.slot_index(), id.slot_index() + 1)
                != 0
        }
        RecordRef::Relation(id) => {
            partition
                .relation_arena
                .live_bitset
                .count_ones_in_range(id.slot_index(), id.slot_index() + 1)
                != 0
        }
    }
}
