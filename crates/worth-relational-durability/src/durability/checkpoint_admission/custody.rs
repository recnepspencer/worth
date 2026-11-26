//! Immutable captured provenance and each receiver's actual retirement edge.
use worth_relational_branch::internal_port as branch;
use worth_relational_contracts::internal_port as contracts;
use branch::durability::checkpoint_admission::row_custody::{AdmissionCustody, SelectedCheckpointSource};
use branch::durability::checkpoint_admission::admitted_commit::CheckpointAdmittedCommit;
use contracts::durability::checkpoint_admission::refusal::RelationalCheckpointAdmissionRefusal as Refusal;
use crate::durability::data::NativeCheckpointDirectoryEdge;
use worth_relational_storage::internal_port::storage::data::directory_source_edge::NativeDirectoryRetirementHandoff;
use crate::durability::data::RelationalCheckpointDirectory;
use worth_relational_storage::internal_port::durability::medium::checkpoint_acknowledgement_denial::RelationalCheckpointAcknowledgementDenial as Ack;
use branch::history::commit::index_entry::RelationalCommitIndexEntry;
use std::sync::{Arc, Weak};
use worth_execution::{ExecutionOrderedMap, ExecutionRequest};

#[derive(Debug, Clone)]
pub struct CheckpointAdmissionOwner(Arc<AdmissionCustody>);
#[derive(Debug, Clone)]
pub struct CheckpointAdmissionBinding {
    owner: CheckpointAdmissionOwner,
    directory: NativeCheckpointDirectoryEdge,
}
impl worth_proof::AuthorityMarker for CheckpointAdmissionBinding {}

/// Directory installation publishes the row owner and its source together.
#[derive(Debug)]
pub(crate) struct DirectoryAdmissionIdentity {
    selected: Arc<SelectedCheckpointSource>,
    owner: Weak<AdmissionCustody>,
}

impl DirectoryAdmissionIdentity {
    fn owner(&self) -> Option<Arc<AdmissionCustody>> {
        self.owner
            .upgrade()
            .filter(|owner| owner.belongs_to(&self.selected))
    }
}
impl CheckpointAdmissionOwner {
    pub fn bind(
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
        let selected = SelectedCheckpointSource::retain(&directory.source, request)?;
        let owner = AdmissionCustody::retain(&selected, request)?;
        let mut rows = ExecutionOrderedMap::default();
        let index = directory.commit_index();
        for (ordinal, reference) in index.descriptor().pages.iter().enumerate() {
            let page = index.page(ordinal).map_err(|cause| {
                contracts::durability::checkpoint_admission::refusal::native_stop(
                    crate::durability::data::commit_index_access::stop(reference.first, cause),
                )
            })?;
            for entry in &page.page.rows {
                let row =
                    super::admit_row::admit_row(directory, entry, Arc::downgrade(&owner), request)?;
                rows.insert_cloned_native_for_request(&entry.commit.commit_id, &row, *request)
                    .map_err(|cause| {
                        contracts::durability::checkpoint_admission::refusal::native_stop(
                            contracts::durability::data::recovery_stop::map_stop(cause),
                        )
                    })?;
            }
        }
        owner.install_rows(rows)?;
        directory
            .admission_owner
            .set(DirectoryAdmissionIdentity {
                selected,
                owner: Arc::downgrade(&owner),
            })
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
            .and_then(DirectoryAdmissionIdentity::owner)
            .ok_or(Refusal::OwnerUnavailable)?;
        let count = owner.count()?;
        let levels = u64::from(usize::BITS - count.leading_zeros()) + 1;
        let bytes =
            branch::history::commit::retained_index::RetainedCanonicalIndex::allocation_bytes(
                &entry.commit,
            )
            .ok_or(Refusal::CapacityOverflow)?;
        let units = levels
            .checked_mul(22)
            .and_then(|n| n.checked_add(bytes))
            .ok_or(Refusal::CapacityOverflow)?;
        contracts::durability::data::recovery_stop::account_work(request, units)
            .map_err(contracts::durability::checkpoint_admission::refusal::native_stop)?;
        owner.commit(entry, directory.manifest.covered_position)
    }
}
impl CheckpointAdmissionBinding {
    pub(super) fn capture_directory(&self) -> Result<Arc<RelationalCheckpointDirectory>, Refusal> {
        let directory = self.directory.snapshot().ok_or(Refusal::OwnerUnavailable)?;
        if !directory
            .admission_owner
            .get()
            .and_then(DirectoryAdmissionIdentity::owner)
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
            .and_then(DirectoryAdmissionIdentity::owner)
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
        source: &Arc<dyn worth_relational_storage::internal_port::durability::medium::cold_source::RelationalDurableColdSource>,
    ) -> bool {
        self.directory
            .snapshot()
            .is_some_and(|directory| Arc::ptr_eq(&directory.source, source))
            && commit.belongs_to(&self.owner.0)
    }
    pub(crate) fn preflight_retirement(
        &self,
        source: &dyn worth_relational_storage::internal_port::durability::medium::cold_source::RelationalDurableColdSource,
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
