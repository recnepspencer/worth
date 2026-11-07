//! A fully funded, single-use batch bound to its actual prepared canonical root.
use super::super::prepared_generation;
use super::{publication::PreparedRefresh, work::MaintenanceWork};
use crate::branch::RelationalBranchRoot;
use crate::history::data::BranchId;
use crate::indexes::data::{DerivedIndexGeneration, DerivedIndexMaintenanceDenialKind as Denial};
use std::sync::Arc;
/// Fully derived payload owned by one single-use Relational candidate. Only
/// catalog installation remains after its canonical branch movement.
pub(crate) struct PreparedCandidateIndexPublication {
    pub(super) prepared: CandidateIndexPreparation,
    pub(super) funding: worth_execution::ExecutionMemoryReservation,
    pub(super) work: crate::indexes::data::DerivedIndexMaintenanceWork,
}

pub(super) struct CandidateIndexPreparation {
    root_id: u64,
    branch_id: BranchId,
    commit_id: crate::history::data::CommitId,
    version_id: crate::identity::data::VersionId,
    schema_version: crate::schema::data::SchemaVersionId,
    publication: crate::runtime::PreparedGenerationBatch,
}

impl PreparedCandidateIndexPublication {
    pub(crate) fn work(&self) -> crate::indexes::data::DerivedIndexMaintenanceWork {
        self.work
    }

    pub(crate) fn matches_root(&self, root: &Arc<RelationalBranchRoot>) -> bool {
        root.id() == self.prepared.root_id
            && root.commit_id() == Some(self.prepared.commit_id)
            && root.canonical_envelope().is_some_and(|envelope| {
                envelope.branch_context == self.prepared.branch_id
                    && envelope.commit.version_id == self.prepared.version_id
            })
            && root.schema_authority().schema_version() == self.prepared.schema_version
    }

    pub(crate) fn publish(self, _runtime: &crate::runtime::RelationalRuntime) {
        let funding = self.funding;
        self.prepared.publication.publish();
        drop(funding);
    }
}

impl CandidateIndexPreparation {
    pub(super) fn prepare(
        indexes: &crate::runtime::IndexingSubsystem,
        prepared_root: &Arc<RelationalBranchRoot>,
        prepared: PreparedRefresh,
        work: &mut MaintenanceWork,
    ) -> Result<Self, Denial> {
        work.charge(prepared.entries.len())?;
        work.reserve_generation_publications(prepared.entries.len());
        work.array::<DerivedIndexGeneration>(prepared.entries.len())?;
        work.array::<crate::indexes::data::DerivedIndexGenerationId>(prepared.entries.len())?;
        work.prepare(
            0,
            (prepared.publication.branch_id.0.len() * 2 * prepared.entries.len()) as u64,
        )?;
        let generation_ids = indexes
            .reserve_generation_ids(prepared.entries.len())
            .ok_or(Denial::GenerationIdentityExhausted)?;
        let generations = prepared
            .entries
            .into_iter()
            .zip(generation_ids)
            .map(|((index_id, entries), generation_id)| {
                prepared_generation(&prepared.publication, generation_id, index_id, entries)
            })
            .collect();
        let publication = indexes
            .prepare_generation_batch(generations, prepared.basis, |units, bytes| {
                work.prepare(units, bytes)
            })
            .map_err(|stop| match stop {
                crate::indexes::data::SelectedIndexGenerationAdmissionStop::Admission(stop) => stop,
                crate::indexes::data::SelectedIndexGenerationAdmissionStop::AccountingOverflow => {
                    Denial::WorkBudgetExceeded
                }
            })?;
        Ok(CandidateIndexPreparation {
            root_id: prepared_root.id(),
            branch_id: prepared_root
                .canonical_envelope()
                .ok_or(Denial::CommitMismatch)?
                .branch_context
                .clone(),
            commit_id: prepared_root.commit_id().ok_or(Denial::CommitMismatch)?,
            version_id: prepared_root
                .axes()
                .map(|axes| crate::identity::data::VersionId(axes.storage_version))
                .ok_or(Denial::CommitMismatch)?,
            schema_version: prepared_root.schema_authority().schema_version(),
            publication,
        })
    }
}
