//! Read owner-resident original meaning without claiming durable source custody.
use super::{DurabilityAuthority, RelationalAssociatedEffectRefusal as Refusal};
use crate::history::data::{PositionedCanonicalCommit, RelationalCommitReceipt};
use crate::transactions::data::RecoveredRelationalSemanticResult;

/// The actual resident Native owner artifact. This cannot qualify a directory
/// Home, mint a performed effect, or reconstruct a live CommitResult.
#[derive(Debug)]
pub struct ObservedRelationalMemoryCommit {
    positioned: PositionedCanonicalCommit,
    semantic: RecoveredRelationalSemanticResult,
    digest: [u8; 32],
}

impl ObservedRelationalMemoryCommit {
    pub fn receipt(&self) -> &RelationalCommitReceipt {
        self.semantic.semantic().receipt()
    }
    pub fn semantic_result(&self) -> &RecoveredRelationalSemanticResult {
        &self.semantic
    }
    pub fn position(&self) -> crate::publication::patch::data::PatchStreamPosition {
        self.positioned.position()
    }
    pub fn artifact_digest(&self) -> &[u8; 32] {
        &self.digest
    }

    /// An opaque admitted basis must retain the SAME canonical original,
    /// rather than merely repeating another runtime's scalar coordinates.
    pub fn matches_original_basis(
        &self,
        basis: &crate::branch::AdmittedRelationalBranchBasis,
    ) -> bool {
        basis
            .inner
            .root
            .canonical_envelope()
            .is_some_and(|original| {
                std::sync::Arc::ptr_eq(original, self.positioned.canonical_arc())
            })
    }

    pub(crate) fn matches_original_envelope(
        &self,
        expected: &std::sync::Weak<crate::history::data::CanonicalCommitEnvelope>,
    ) -> bool {
        std::sync::Weak::ptr_eq(
            expected,
            &std::sync::Arc::downgrade(self.positioned.canonical_arc()),
        )
    }

    pub fn created_entity(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        entity: crate::identity::data::EntityId,
    ) -> Result<Option<super::ReadmittedRelationalCreatedEntity<'_>>, Refusal> {
        super::created_row::read_created_entity(
            execution,
            self.semantic.semantic(),
            self.positioned.envelope(),
            entity,
        )
    }
}

impl DurabilityAuthority<'_> {
    /// Observe a resident artifact explicitly. A source-backed artifact must
    /// use its source-readmission door; this is never a durability fallback.
    pub fn observe_memory_commit(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        receipt: &RelationalCommitReceipt,
    ) -> Result<ObservedRelationalMemoryCommit, Refusal> {
        crate::execution::check_request(execution).map_err(|cause| {
            Refusal::Recovery(crate::durability::data::RelationalRecoveryStop::Request(
                cause,
            ))
        })?;
        let _operation = self
            .runtime
            .owner_binding()
            .admit()
            .ok_or(Refusal::OwnerUnavailable)?;
        let artifact = self
            .runtime
            .history
            .commit_artifact(receipt.commit_id)
            .ok_or(Refusal::SubjectMismatch)?;
        if artifact.receipt() != *receipt {
            return Err(Refusal::SubjectMismatch);
        }
        let residency = artifact.envelope_residency();
        if residency.source_placement().is_some() {
            return Err(Refusal::SubjectMismatch);
        }
        let positioned = residency
            .demand_positioned(execution, 1 << 30)
            .map_err(Refusal::Recovery)?;
        if &positioned.envelope().commit != receipt {
            return Err(Refusal::SubjectMismatch);
        }
        let digest = crate::durability::canonical_commit_artifact::RelationalCanonicalCommitArtifact::envelope_payload_digest_for_request(
            positioned.position(), positioned.canonical_arc(), execution,
        ).map_err(Refusal::Recovery)?;
        let semantic = RecoveredRelationalSemanticResult {
            semantic: positioned.envelope().semantic_result_arc(),
        };
        Ok(ObservedRelationalMemoryCommit {
            positioned,
            semantic,
            digest,
        })
    }
}
