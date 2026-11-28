use crate::durability::canonical_commit_artifact::RelationalCanonicalCommitArtifact as Artifact;
use crate::runtime::state::subsystems::durability::checkpoint_selection::RelationalCheckpointManifestSelection;
use contracts::branch::recorded_basis::refusal::RelationalRecordedComponentRefusal as Refusal;
use worth_foundational::FoundationalBranchTarget;
use worth_relational_branch::internal_port::branch::basis::{
    AdmittedRelationalBranchBasis, AdmittedRelationalBranchBasisFieldsPort as _,
};
use worth_relational_branch::internal_port::branch::recorded_basis::RecordedRelationalComponentReferenceOwnerPort as _;
use worth_relational_branch::internal_port::branch::recorded_basis::{
    copying, RecordedRelationalComponentReference,
};
use worth_relational_branch::internal_port::history::data::canonical_commit_envelope::CanonicalCommitEnvelopeBodyOwnerPort as _;
use worth_relational_branch::internal_port::runtime::state::subsystems::history::HistorySubsystem;
use worth_relational_contracts::internal_port as contracts;

pub fn export_retained_component(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    basis: &AdmittedRelationalBranchBasis,
    runtime_instance_id: u64,
    history: &HistorySubsystem,
    checkpoint_selection: &RelationalCheckpointManifestSelection,
    performed_envelope: impl FnOnce(
        contracts::history::data::coordinates::CommitId,
    ) -> Option<
        std::sync::Arc<worth_relational_branch::internal_port::history::commit::envelope_residency::RelationalCommitEnvelopeResidency>,
    >,
) -> Result<RecordedRelationalComponentReference, Refusal> {
    contracts::execution::request_custody::check_request(execution).map_err(Refusal::Request)?;
    worth_relational_branch::internal_port::branch::basis_identity_validation::require_local_branch_identity(
        runtime_instance_id,
        history,
        basis.identity(),
    )
    .map_err(Refusal::Basis)?;
    let target = match basis.reference().target() {
        FoundationalBranchTarget::Empty => None,
        FoundationalBranchTarget::Basis(target) => {
            let commit_id =
                contracts::history::data::coordinates::CommitId(target.selected_commit_id());
            let cell = history
                .commit_artifact(commit_id)
                .map(|artifact| artifact.envelope_residency())
                .or_else(|| performed_envelope(commit_id))
                .ok_or(Refusal::MissingPerformedCommit { commit_id })?;
            if let Some(worth_relational_branch::internal_port::branch::root::commit_evidence::RelationalRootCommitEvidence::Checkpoint(commit)) =
                basis.owner_inner().root.commit_evidence()
            {
                let entry = commit.index();
                let (actual, source) = cell
                    .source_placement()
                    .ok_or(Refusal::CanonicalMismatch { commit_id })?;
                if !checkpoint_selection.matches_admitted_source(commit, &source)
                    || actual.as_ref() != entry
                    || entry.commit.commit_id != commit_id
                    || entry.commit.version_id.0 != target.version_id()
                    || basis.owner_inner().root.descriptor() != Some(target.roots())
                {
                    return Err(Refusal::CanonicalMismatch { commit_id });
                }
                return RecordedRelationalComponentReference::copy_original(
                    execution,
                    basis.descriptor(),
                    Some(copying::CanonicalDescription {
                        receipt: &entry.commit,
                        position: entry.position,
                        artifact_digest: entry.payload_digest,
                        roots: &entry.root,
                    }),
                );
            }
            let positioned = if cell.source_placement().is_some() {
                crate::durability::history_source_readmission::readmit_original_positioned(
                    &cell, execution,
                )
            } else {
                cell.demand_positioned(execution, 1 << 30)
            }
            .map_err(Refusal::Source)?;
            let envelope = basis
                .owner_inner()
                .root
                .canonical_envelope()
                .ok_or(Refusal::CanonicalMismatch { commit_id })?;
            if positioned.commit.commit_id != commit_id
                || positioned.commit.version_id.0 != target.version_id()
                || basis.owner_inner().root.descriptor() != Some(target.roots())
                || !Artifact::matches_positioned_payload_for_request(
                    &positioned,
                    envelope,
                    execution,
                )
                .map_err(Refusal::Source)?
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
                None => worth_relational_branch::internal_port::history::commit::payload_commitment::digest_envelope_for_request(
                    positioned.position(),
                    positioned.envelope(),
                    execution,
                )
                .map_err(Refusal::Source)?,
            };
            Some(copying::CanonicalDescription {
                receipt: &envelope.commit,
                position: positioned.position(),
                artifact_digest,
                roots: target.roots(),
            })
        }
    };
    contracts::execution::request_custody::check_request(execution).map_err(Refusal::Request)?;
    RecordedRelationalComponentReference::copy_original(execution, basis.descriptor(), target)
}
