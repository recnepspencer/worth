//! Immutable captured provenance and each receiver's actual retirement edge.
use super::{CheckpointAdmittedCommit, RelationalCheckpointAdmissionRefusal as Refusal};
use crate::durability::data::{
    NativeCheckpointDirectoryEdge, NativeDirectoryRetirementHandoff, RelationalCheckpointDirectory,
};
use crate::durability::medium::RelationalCheckpointAcknowledgementDenial as Ack;
use crate::history::{data::CommitId, RelationalCommitIndexEntry};
use std::sync::{Arc, OnceLock, Weak};
use worth_execution::{ExecutionMemoryReservation, ExecutionOrderedMap, ExecutionRequest};

#[derive(Debug, Clone)]
pub(crate) struct CheckpointAdmissionOwner(Arc<AdmissionCustody>);
#[derive(Debug, Clone)]
pub(crate) struct CheckpointAdmissionBinding {
    owner: CheckpointAdmissionOwner,
    directory: NativeCheckpointDirectoryEdge,
}
impl worth_proof::AuthorityMarker for CheckpointAdmissionBinding {}

#[derive(Debug)]
pub(crate) struct AdmissionCustody {
    pub(super) directory: Weak<RelationalCheckpointDirectory>,
    rows: OnceLock<ExecutionOrderedMap<CommitId, CheckpointAdmittedCommit>>,
    _memory: Option<ExecutionMemoryReservation>,
}
impl CheckpointAdmissionOwner {
    pub(crate) fn bind(
        directory: &Arc<RelationalCheckpointDirectory>,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<worth_proof::RelationalReadmitted<CheckpointAdmissionBinding>, Refusal> {
        if directory.admission_owner.get().is_some() {
            return Err(Refusal::OwnerUnavailable);
        }
        directory
            .admission_image
            .as_ref()
            .ok_or(Refusal::MissingArtifact)?;
        let layout = std::alloc::Layout::new::<[usize; 2]>()
            .extend(std::alloc::Layout::new::<AdmissionCustody>())
            .map_err(|_| Refusal::CapacityOverflow)?
            .0
            .pad_to_align();
        let memory = request
            .reserve_retained_memory(layout.size() as u64)
            .map_err(|cause| Refusal::Request(cause.into()))?;
        let owner = Arc::new(AdmissionCustody {
            directory: Arc::downgrade(directory),
            rows: Default::default(),
            _memory: memory,
        });
        let mut rows = ExecutionOrderedMap::default();
        for entry in &directory.manifest.commits {
            let row = CheckpointAdmittedCommit::prepare(
                directory,
                entry,
                Arc::downgrade(&owner),
                request,
            )?;
            rows.insert_cloned_native_for_request(&entry.commit.commit_id, &row, *request)
                .map_err(|cause| {
                    super::native_stop(crate::durability::checkpoint_history::capture::map_stop(
                        cause,
                    ))
                })?;
        }
        owner
            .rows
            .set(rows)
            .map_err(|_| Refusal::OwnerUnavailable)?;
        directory
            .admission_owner
            .set(Arc::downgrade(&owner))
            .map_err(|_| Refusal::OwnerUnavailable)?;
        Ok(worth_proof::RelationalReadmitted::from_owner(
            CheckpointAdmissionBinding {
                owner: Self(owner),
                directory: NativeCheckpointDirectoryEdge::new(directory.clone()),
            },
        ))
    }
    pub(super) fn owns(&self, origin: &Weak<AdmissionCustody>) -> bool {
        origin
            .upgrade()
            .is_some_and(|origin| Arc::ptr_eq(&self.0, &origin))
    }
}
impl CheckpointAdmissionBinding {
    pub(crate) fn commit(
        &self,
        entry: &RelationalCommitIndexEntry,
    ) -> Result<Option<CheckpointAdmittedCommit>, Refusal> {
        let directory = self.directory.snapshot().ok_or(Refusal::OwnerUnavailable)?;
        let row = self
            .owner
            .0
            .rows
            .get()
            .ok_or(Refusal::OwnerUnavailable)?
            .get(&entry.commit.commit_id);
        match row {
            Some(row) if row.index() == entry => Ok(Some(row.clone())),
            Some(_) => Err(Refusal::InvalidProjectedMeaning),
            None if entry.position > directory.manifest.covered_position => Ok(None),
            None => Err(Refusal::InvalidProjectedMeaning),
        }
    }
    pub(crate) fn matches_source(
        &self,
        commit: &CheckpointAdmittedCommit,
        source: &Arc<dyn crate::durability::medium::RelationalDurableColdSource>,
    ) -> bool {
        self.directory
            .snapshot()
            .is_some_and(|directory| Arc::ptr_eq(&directory.source, source))
            && commit.belongs_to(&self.owner)
    }
    pub(crate) fn preflight_retirement(
        &self,
        source: &dyn crate::durability::medium::RelationalDurableColdSource,
    ) -> Result<(), Ack> {
        if let Some(directory) = self.directory.snapshot() {
            directory.preflight_retirement(source)?;
        }
        Ok(())
    }
    pub(crate) fn loan_source_retirement(
        &mut self,
        handoff: &mut Option<NativeDirectoryRetirementHandoff>,
    ) -> Result<bool, Ack> {
        self.directory.loan(handoff)
    }
    pub(crate) fn return_source_retirement(
        &mut self,
        handoff: &mut Option<NativeDirectoryRetirementHandoff>,
    ) -> Result<bool, Ack> {
        self.directory.return_handoff(handoff)
    }
}
