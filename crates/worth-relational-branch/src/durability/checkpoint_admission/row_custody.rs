//! Selected directory provider and per-row custody at the native commit boundary.
use super::admitted_commit::CheckpointAdmittedCommit;
use super::refusal::RelationalCheckpointAdmissionRefusal as Refusal;
use crate::durability::medium::RelationalDurableColdSource;
use crate::history::{commit::RelationalCommitIndexEntry, data::CommitId};
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex, Weak};
use worth_execution::{ExecutionMemoryReservation, ExecutionRequest};

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
    count: usize,
    rows: Mutex<BTreeMap<CommitId, CheckpointAdmittedCommit>>,
    _memory: Option<ExecutionMemoryReservation>,
}

impl AdmissionCustody {
    pub fn retain(
        source: &Arc<SelectedCheckpointSource>,
        count: usize,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<Arc<Self>, Refusal> {
        let memory = request
            .reserve_retained_memory(header_bytes::<Self>()?)
            .map_err(|cause| Refusal::Request(cause.into()))?;
        Ok(Arc::new(Self {
            source: Arc::downgrade(source),
            count,
            rows: Mutex::new(BTreeMap::new()),
            _memory: memory,
        }))
    }

    pub fn cache_row(
        &self,
        commit_id: CommitId,
        row: CheckpointAdmittedCommit,
    ) -> CheckpointAdmittedCommit {
        let mut rows = self
            .rows
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        rows.entry(commit_id).or_insert(row).clone()
    }

    pub fn count(&self) -> Result<usize, Refusal> {
        Ok(self.count)
    }

    pub fn belongs_to(&self, selection: &Arc<SelectedCheckpointSource>) -> bool {
        Weak::ptr_eq(&self.source, &Arc::downgrade(selection))
    }

    pub(super) fn selects_source(&self, source: &Arc<dyn RelationalDurableColdSource>) -> bool {
        self.source
            .upgrade()
            .is_some_and(|selection| Weak::ptr_eq(&selection.source, &Arc::downgrade(source)))
    }

    pub fn cached_commit(
        &self,
        entry: &RelationalCommitIndexEntry,
    ) -> Result<Option<CheckpointAdmittedCommit>, Refusal> {
        let rows = self
            .rows
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        match rows.get(&entry.commit.commit_id) {
            Some(row) if row.index() == entry => Ok(Some(row.clone())),
            Some(_) => Err(Refusal::InvalidProjectedMeaning),
            None => Ok(None),
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
