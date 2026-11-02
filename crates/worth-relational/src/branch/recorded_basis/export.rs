use super::{
    copying, RecordedRelationalComponentReference, RelationalRecordedComponentRefusal as Refusal,
};
use crate::branch::AdmittedRelationalBranchBasis;
use crate::durability::canonical_commit_artifact::RelationalCanonicalCommitArtifact as Artifact;
use crate::runtime::RelationalRuntime;
use worth_foundational::FoundationalBranchTarget;

impl RelationalRuntime {
    /// Export descriptions of one actual current subject. The artifact digest
    /// does not qualify memory backing as durable source evidence.
    pub(crate) fn export_recorded_current_component(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        basis: &AdmittedRelationalBranchBasis,
    ) -> Result<RecordedRelationalComponentReference, Refusal> {
        crate::execution::check_request(execution).map_err(Refusal::Request)?;
        self.compare_current_exact(basis).map_err(Refusal::Basis)?;
        let recorded = self.export_recorded_retained_component(execution, basis)?;
        self.compare_current_exact(basis).map_err(Refusal::Basis)?;
        Ok(recorded)
    }

    /// Record the exact subject already pinned by this actual admitted basis,
    /// even after another publication advances the branch. No descriptor can
    /// be paired with a different result or selected by current-head fallback.
    pub(crate) fn export_recorded_retained_component(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        basis: &AdmittedRelationalBranchBasis,
    ) -> Result<RecordedRelationalComponentReference, Refusal> {
        crate::execution::check_request(execution).map_err(Refusal::Request)?;
        super::super::basis_identity_validation::require_local_branch_identity(
            self,
            basis.identity(),
        )
        .map_err(Refusal::Basis)?;
        let target = match basis.reference().target() {
            FoundationalBranchTarget::Empty => None,
            FoundationalBranchTarget::Basis(target) => {
                let commit_id = crate::history::data::CommitId(target.selected_commit_id());
                let cell = self
                    .history
                    .commit_artifact(commit_id)
                    .map(|artifact| artifact.envelope_residency())
                    .or_else(|| self.history.performed_envelope_residency(commit_id))
                    .ok_or(Refusal::MissingPerformedCommit { commit_id })?;
                let positioned = if cell.source_placement().is_some() {
                    cell.readmit_original_positioned(execution)
                } else {
                    cell.demand_positioned(execution, 1 << 30)
                }
                .map_err(Refusal::Source)?;
                let envelope = basis
                    .inner
                    .root
                    .canonical_envelope()
                    .ok_or(Refusal::CanonicalMismatch { commit_id })?;
                if positioned.commit.commit_id != commit_id
                    || positioned.commit.version_id.0 != target.version_id()
                    || basis.inner.root.descriptor() != Some(target.roots())
                    || !Artifact::matches_positioned_payload(&positioned, envelope).map_err(
                        |error| Refusal::PayloadEncoding {
                            detail: error.to_string(),
                        },
                    )?
                {
                    return Err(Refusal::CanonicalMismatch { commit_id });
                }
                let artifact_digest = match cell.journal_index() {
                    Some(entry) => {
                        if entry.commit != envelope.commit
                            || entry.association_discovery != envelope.association_discovery()
                            || entry.position != positioned.position()
                            || &entry.root != target.roots()
                        {
                            return Err(Refusal::CanonicalMismatch { commit_id });
                        }
                        entry.payload_digest
                    }
                    None => Artifact::positioned_payload_digest(&positioned).map_err(|error| {
                        Refusal::PayloadEncoding {
                            detail: error.to_string(),
                        }
                    })?,
                };
                Some(copying::CanonicalDescription {
                    receipt: &envelope.commit,
                    position: positioned.position(),
                    artifact_digest,
                    roots: target.roots(),
                })
            }
        };
        crate::execution::check_request(execution).map_err(Refusal::Request)?;
        RecordedRelationalComponentReference::copy_original(execution, basis.descriptor(), target)
    }
}
