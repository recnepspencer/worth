//! The generic maintained contract selects journal records, never arena history.
use super::{membership::key, *};
use crate::storage::overlay::{PartitionMutationJournal, PartitionState};
use crate::storage::substrate::maintenance::{
    ChangeWork, MaintenanceRefusal, PartitionChange, RecordChangeState,
};
use crate::storage::substrate::{RecordArena, RecordKind};
use crate::transactions::data::primitives::RecordRef;
use worth_relational_contracts::internal_port::storage::substrate::maintenance::work::ChangeWorkOwnerPort as _;

#[derive(Debug)]
pub enum DirectoryError {
    Admission(MaintenanceRefusal),
    Corrupt,
    /// A cold directory page could not be read.
    Unavailable(crate::storage::data::RelationalStorageUnavailableCause),
}
impl From<MaintenanceRefusal> for DirectoryError {
    fn from(cause: MaintenanceRefusal) -> Self {
        Self::Admission(cause)
    }
}

pub(super) struct DirectoryMaintainer<'a> {
    pub(super) partition: &'a PartitionState,
    pub(super) before: &'a PartitionState,
}
impl DirectoryMaintainer<'_> {
    fn apply_change(
        &mut self,
        previous: &Arc<RegionRecordDirectory>,
        change: &PartitionChange,
        work: &mut ChangeWork<'_>,
    ) -> Result<Arc<RegionRecordDirectory>, DirectoryError> {
        work.prepare_route_lookup(1)?;
        let mut next = previous.as_ref().clone();
        let mut any = false;
        for record in change.records() {
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
        finalize_edit(previous, next, any, &mut |units, bytes| {
            work.prepare(units, bytes).map_err(Into::into)
        })
    }
}
impl RegionRecordDirectory {
    pub fn updated(
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        previous: &Arc<Self>,
        before: &PartitionState,
        partition: &PartitionState,
        journal: &PartitionMutationJournal,
        change: Option<&PartitionChange>,
    ) -> Result<Arc<Self>, DirectoryError> {
        if previous.partition != partition.partition_id {
            return Err(DirectoryError::Corrupt);
        }
        let mut owner = DirectoryMaintainer { partition, before };
        ChangeWork::run_phase(execution, "record_directory", |work| {
            if let Some(change) = change {
                return owner.apply_change(previous, change, work);
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
        finalize_edit(previous, next, any, &mut |units, bytes| {
            work.prepare(units, bytes).map_err(Into::into)
        })
    }
}
/// An edited directory leaves no domain paged: even an aspect-only edit with
/// unchanged live membership must detach from the predecessor's cold cell.
pub(super) fn finalize_edit(
    previous: &Arc<RegionRecordDirectory>,
    mut next: RegionRecordDirectory,
    any: bool,
    prepare: &mut dyn FnMut(u64, u64) -> Result<(), DirectoryError>,
) -> Result<Arc<RegionRecordDirectory>, DirectoryError> {
    if any || next.has_paged_domain() {
        next.entities.make_resident(prepare)?;
        next.relations.make_resident(prepare)?;
        return Ok(Arc::new(next));
    }
    Ok(previous.clone())
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
    pub fn into_failure(
        self,
        partition: PartitionId,
    ) -> crate::storage::residency::RelationalResidencyDemandFailure {
        match self {
            Self::Admission(MaintenanceRefusal::Execution(cause)) => {
                crate::storage::residency::RelationalResidencyDemandFailure::Request(cause)
            }
            Self::Unavailable(cause) => crate::storage::data::RelationalStorageUnavailable {
                subject: crate::storage::data::RelationalStorageSubject::Region(partition),
                cause,
            }
            .into(),
            _ => crate::storage::data::RelationalStorageUnavailable {
                subject: crate::storage::data::RelationalStorageSubject::Region(partition),
                cause: crate::storage::data::RelationalStorageUnavailableCause::IntegrityMismatch,
            }
            .into(),
        }
    }
    pub fn into_content_failure(
        self,
        partition: PartitionId,
    ) -> crate::storage::residency::RegionContentFailure {
        match self {
            Self::Admission(MaintenanceRefusal::Execution(cause)) => {
                crate::storage::residency::RegionContentFailure::Execution(cause)
            }
            Self::Unavailable(cause) => crate::storage::residency::RegionContentFailure::Storage(
                crate::storage::data::RelationalStorageUnavailable {
                    subject: crate::storage::data::RelationalStorageSubject::Region(partition),
                    cause,
                },
            ),
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
