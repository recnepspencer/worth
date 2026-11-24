//! Selected directory provider and per-row custody at the native commit boundary.
use super::admitted_commit::CheckpointAdmittedCommit;
use super::refusal::RelationalCheckpointAdmissionRefusal as Refusal;
use crate::durability::medium::RelationalDurableColdSource;
use crate::history::{commit::RelationalCommitIndexEntry, data::CommitId};
use std::sync::{Arc, OnceLock, Weak};
use worth_execution::{ExecutionMemoryReservation, ExecutionOrderedMap, ExecutionRequest};

/// Lives exactly as long as the admitted serving directory.
#[derive(Debug)]
pub struct SelectedCheckpointSource {
    source: Weak<dyn RelationalDurableColdSource>,
    _memory: Option<ExecutionMemoryReservation>,
}

impl SelectedCheckpointSource {
    pub fn retain(
        source: &Arc<dyn RelationalDurableColdSource>,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<Arc<Self>, Refusal> {
        let memory = request
            .reserve_retained_memory(header_bytes::<Self>()?)
            .map_err(|cause| Refusal::Request(cause.into()))?;
        Ok(Arc::new(Self {
            source: Arc::downgrade(source),
            _memory: memory,
        }))
    }
}

#[derive(Debug)]
pub struct AdmissionCustody {
    source: Weak<SelectedCheckpointSource>,
    rows: OnceLock<ExecutionOrderedMap<CommitId, CheckpointAdmittedCommit>>,
    _memory: Option<ExecutionMemoryReservation>,
}

impl AdmissionCustody {
    pub fn retain(
        source: &Arc<SelectedCheckpointSource>,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<Arc<Self>, Refusal> {
        let memory = request
            .reserve_retained_memory(header_bytes::<Self>()?)
            .map_err(|cause| Refusal::Request(cause.into()))?;
        Ok(Arc::new(Self {
            source: Arc::downgrade(source),
            rows: OnceLock::new(),
            _memory: memory,
        }))
    }

    pub fn install_rows(
        &self,
        rows: ExecutionOrderedMap<CommitId, CheckpointAdmittedCommit>,
    ) -> Result<(), Refusal> {
        self.rows.set(rows).map_err(|_| Refusal::OwnerUnavailable)
    }

    pub fn count(&self) -> Result<usize, Refusal> {
        self.rows
            .get()
            .map(|rows| rows.len())
            .ok_or(Refusal::OwnerUnavailable)
    }

    pub fn belongs_to(&self, selection: &Arc<SelectedCheckpointSource>) -> bool {
        Weak::ptr_eq(&self.source, &Arc::downgrade(selection))
    }

    pub(super) fn selects_source(&self, source: &Arc<dyn RelationalDurableColdSource>) -> bool {
        self.source
            .upgrade()
            .is_some_and(|selection| Weak::ptr_eq(&selection.source, &Arc::downgrade(source)))
    }

    pub fn commit(
        &self,
        entry: &RelationalCommitIndexEntry,
        covered: crate::publication::patch::data::PatchStreamPosition,
    ) -> Result<Option<CheckpointAdmittedCommit>, Refusal> {
        let row = self
            .rows
            .get()
            .ok_or(Refusal::OwnerUnavailable)?
            .get(&entry.commit.commit_id);
        match row {
            Some(row) if row.index() == entry => Ok(Some(row.clone())),
            Some(_) => Err(Refusal::InvalidProjectedMeaning),
            None if entry.position > covered => Ok(None),
            None => Err(Refusal::InvalidProjectedMeaning),
        }
    }
}

fn header_bytes<T>() -> Result<u64, Refusal> {
    Ok(std::alloc::Layout::new::<[usize; 2]>()
        .extend(std::alloc::Layout::new::<T>())
        .map_err(|_| Refusal::CapacityOverflow)?
        .0
        .pad_to_align()
        .size() as u64)
}
