use super::{
    RecordedRelationalCanonicalComponent, RecordedRelationalComponentReference,
    RecordedRelationalComponentTarget, RelationalRecordedComponentRefusal as Refusal,
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
        let descriptor = basis
            .descriptor()
            .in_canonical_frame()
            .map_err(Refusal::Frame)?;
        let target = match basis.reference().target() {
            FoundationalBranchTarget::Empty => RecordedRelationalComponentTarget::Empty,
            FoundationalBranchTarget::Basis(target) => {
                let commit_id = crate::history::data::CommitId(target.selected_commit_id());
                let positioned = self
                    .history
                    .positioned_canonical_commit(commit_id)
                    .ok_or(Refusal::MissingPerformedCommit { commit_id })?;
                let envelope = basis
                    .inner
                    .root
                    .canonical_envelope()
                    .ok_or(Refusal::CanonicalMismatch { commit_id })?;
                if !std::sync::Arc::ptr_eq(positioned.canonical_arc(), envelope)
                    || positioned.commit.commit_id != commit_id
                    || positioned.commit.version_id.0 != target.version_id()
                    || basis.inner.root.descriptor() != Some(target.roots())
                {
                    return Err(Refusal::CanonicalMismatch { commit_id });
                }
                let cell = self
                    .history
                    .performed_envelope_residency(commit_id)
                    .ok_or(Refusal::MissingPerformedCommit { commit_id })?;
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
                RecordedRelationalComponentTarget::Canonical(RecordedRelationalCanonicalComponent {
                    receipt: envelope.commit.clone(),
                    position: positioned.position(),
                    artifact_digest,
                    roots: target.roots().clone(),
                })
            }
        };
        crate::execution::check_request(execution).map_err(Refusal::Request)?;
        self.compare_current_exact(basis).map_err(Refusal::Basis)?;
        Ok(RecordedRelationalComponentReference { descriptor, target })
    }
}
