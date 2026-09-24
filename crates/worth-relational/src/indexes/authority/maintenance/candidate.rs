use std::sync::Arc;

use super::{basis, work::MaintenanceWork};
use crate::branch::RelationalBranchRoot;
use crate::history::data::BranchId;
use crate::indexes::data::{
    DerivedIndexBuildRequest, DerivedIndexGeneration, DerivedIndexId,
    DerivedIndexMaintenanceBudget, DerivedIndexMaintenanceDenial,
    DerivedIndexMaintenanceDenialKind as Denial, DerivedIndexMaintenanceWork,
};
use crate::mvcc::PreparedRelationalCommitCandidate;
use crate::runtime::VisibilityProjectionView;
use crate::snapshots::data::SnapshotHandle;
use crate::visibility::snapshot_states::{SnapshotStateBasis, VisibilitySnapshotBasis};

use super::super::{prepared_generation, IndexAuthority};

/// Fully derived payload owned by one single-use Relational candidate. Only
/// catalog installation remains after its canonical branch movement.
pub(crate) struct PreparedCandidateIndexPublication {
    root_id: u64,
    branch_id: BranchId,
    commit_id: crate::history::data::CommitId,
    version_id: crate::identity::data::VersionId,
    schema_version: crate::schema::data::SchemaVersionId,
    generations: Vec<DerivedIndexGeneration>,
}

impl PreparedCandidateIndexPublication {
    pub(crate) fn matches_root(&self, root: &Arc<RelationalBranchRoot>) -> bool {
        root.id() == self.root_id
            && root.commit_id() == Some(self.commit_id)
            && root.canonical_envelope().is_some_and(|envelope| {
                envelope.branch_context == self.branch_id
                    && envelope.commit.version_id == self.version_id
            })
            && root.schema_authority().schema_version() == self.schema_version
    }

    pub(crate) fn publish(self, runtime: &crate::runtime::RelationalRuntime) {
        for generation in self.generations {
            runtime.indexes.publish_generation(generation);
        }
    }
}

impl IndexAuthority<'_> {
    /// Budget and derive exact native index deltas from a prepared candidate
    /// before World can perform its publication. No generation is made visible
    /// until the candidate's performed-commit finalizer consumes this payload.
    pub fn prepare_for_candidate(
        &self,
        candidate: &mut PreparedRelationalCommitCandidate,
        index_ids: &[DerivedIndexId],
        before: Option<&SnapshotHandle>,
        budget: DerivedIndexMaintenanceBudget,
    ) -> Result<DerivedIndexMaintenanceWork, DerivedIndexMaintenanceDenial> {
        let mut work = MaintenanceWork::new(budget);
        let result = self.prepare_candidate_indexes(candidate, index_ids, before, &mut work);
        result.map(|()| work.counts).map_err(|kind| work.deny(kind))
    }

    fn prepare_candidate_indexes(
        &self,
        candidate: &mut PreparedRelationalCommitCandidate,
        index_ids: &[DerivedIndexId],
        before: Option<&SnapshotHandle>,
        work: &mut MaintenanceWork,
    ) -> Result<(), Denial> {
        if candidate.runtime_instance_id() != self.runtime.runtime_instance_id() {
            return Err(Denial::ForeignCandidate);
        }
        if candidate.lifetime_expired() {
            return Err(Denial::CandidateUnavailable);
        }
        if candidate.indexes_already_prepared() {
            return Err(Denial::CandidateIndexesAlreadyPrepared);
        }
        work.charge(index_ids.len())?;
        let (expected_root, prepared_root) = candidate
            .index_preparation_roots()
            .ok_or(Denial::CandidateUnavailable)?;
        let branch_id = candidate.branch().clone();
        let envelope = prepared_root
            .canonical_envelope()
            .ok_or(Denial::CommitMismatch)?;
        if envelope.branch_context != branch_id {
            return Err(Denial::CommitMismatch);
        }
        let request = DerivedIndexBuildRequest {
            source_commit_id: envelope.commit.commit_id,
            branch_id: branch_id.clone(),
            index_ids: index_ids.to_vec(),
        };
        let after =
            prepared_projection(self.runtime, branch_id.clone(), Arc::clone(&prepared_root));
        let before = before
            .map(|snapshot| {
                if snapshot.branch_id() != &request.branch_id {
                    return Err(Denial::BeforeRootMismatch);
                }
                self.runtime
                    .read_truth()
                    .project_snapshot(snapshot)
                    .ok_or(Denial::SnapshotUnavailable)
            })
            .transpose()?;
        validate_expected_root(&expected_root, before.as_ref())?;
        basis::validate_prepared(&request, before.as_ref(), &after)?;
        let prepared = self.prepare_refresh(&request, before.as_ref(), &after, work)?;
        work.charge(prepared.entries.len())?;
        work.counts.generation_publications_reserved = prepared.entries.len();
        let generation_ids = self
            .runtime
            .indexes
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
        candidate.attach_prepared_indexes(PreparedCandidateIndexPublication {
            root_id: prepared_root.id(),
            branch_id,
            commit_id: request.source_commit_id,
            version_id: after.version_id(),
            schema_version: prepared_root.schema_authority().schema_version(),
            generations,
        })
    }
}

fn prepared_projection(
    runtime: &crate::runtime::RelationalRuntime,
    branch_id: BranchId,
    root: Arc<RelationalBranchRoot>,
) -> VisibilityProjectionView<'_> {
    VisibilityProjectionView::new(
        runtime,
        SnapshotStateBasis::Exact(VisibilitySnapshotBasis::from_prepared_root(branch_id, root)),
    )
}

fn validate_expected_root(
    expected: &Arc<RelationalBranchRoot>,
    before: Option<&VisibilityProjectionView<'_>>,
) -> Result<(), Denial> {
    if let Some(before) = before {
        let selected = before.selected_root().ok_or(Denial::BeforeRootMismatch)?;
        if selected.id() != expected.id() {
            return Err(Denial::BeforeRootMismatch);
        }
    }
    Ok(())
}
