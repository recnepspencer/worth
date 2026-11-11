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
    pub(crate) fn read_existing_commit(
        directory: &Arc<RelationalCheckpointDirectory>,
        entry: &RelationalCommitIndexEntry,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<Option<CheckpointAdmittedCommit>, Refusal> {
        let owner = directory
            .admission_owner
            .get()
            .and_then(Weak::upgrade)
            .ok_or(Refusal::OwnerUnavailable)?;
        if !owner
            .directory
            .upgrade()
            .is_some_and(|actual| Arc::ptr_eq(&actual, directory))
        {
            return Err(Refusal::OwnerUnavailable);
        }
        let count = owner.rows.get().ok_or(Refusal::OwnerUnavailable)?.len();
        let levels = u64::from(usize::BITS - count.leading_zeros()) + 1;
        let bytes = crate::history::RetainedCanonicalIndex::allocation_bytes(&entry.commit)
            .ok_or(Refusal::CapacityOverflow)?;
        let units = levels
            .checked_mul(22)
            .and_then(|n| n.checked_add(bytes))
            .ok_or(Refusal::CapacityOverflow)?;
        crate::durability::artifact_payload_emission::request::account_work(request, units)
            .map_err(super::native_stop)?;
        owner.commit(entry, directory.manifest.covered_position)
    }
    pub(super) fn owns(&self, origin: &Weak<AdmissionCustody>) -> bool {
        origin
            .upgrade()
            .is_some_and(|origin| Arc::ptr_eq(&self.0, &origin))
    }
}
impl CheckpointAdmissionBinding {
    pub(crate) fn require_image(
        &self,
        expected: &super::RelationalCheckpointAdmissionArtifact,
    ) -> Result<Arc<RelationalCheckpointDirectory>, Refusal> {
        let directory = self.capture_directory()?;
        if !directory
            .admission_image
            .as_ref()
            .is_some_and(|image| image.same_owner(expected))
        {
            return Err(Refusal::MissingCaptureOrigin);
        }
        Ok(directory)
    }
    pub(super) fn capture_directory(&self) -> Result<Arc<RelationalCheckpointDirectory>, Refusal> {
        let directory = self.directory.snapshot().ok_or(Refusal::OwnerUnavailable)?;
        if !directory
            .admission_owner
            .get()
            .and_then(Weak::upgrade)
            .is_some_and(|owner| Arc::ptr_eq(&owner, &self.owner.0))
        {
            return Err(Refusal::OwnerUnavailable);
        }
        Ok(directory)
    }

    pub(crate) fn commit(
        &self,
        entry: &RelationalCommitIndexEntry,
    ) -> Result<Option<CheckpointAdmittedCommit>, Refusal> {
        let directory = self.directory.snapshot().ok_or(Refusal::OwnerUnavailable)?;
        if !directory
            .admission_owner
            .get()
            .and_then(Weak::upgrade)
            .is_some_and(|actual| Arc::ptr_eq(&actual, &self.owner.0))
        {
            return Err(Refusal::OwnerUnavailable);
        }
        self.owner
            .0
            .commit(entry, directory.manifest.covered_position)
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

impl AdmissionCustody {
    fn commit(
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
