//! Evidence retained from the exact memory-owned canonical commit.
use crate::internal_port::history::data::canonical_commit_envelope::CanonicalCommitEnvelopeOwnerPort as _;
use worth_relational_contracts::internal_port::transactions::data::outcomes::commit_semantic_result::assemble_recovered_relational_semantic_result;
use super::created_row_read::{read_created_entity, ReadmittedRelationalCreatedEntity};
use super::refusal::RelationalAssociatedEffectRefusal as Refusal;
use crate::history::data::canonical_commit_envelope::shared::SharedCanonicalCommitEnvelopeOwnerPort as _;
use crate::history::data::{
    PositionedCanonicalCommit, RelationalCommitReceipt, SharedCanonicalCommitEnvelope,
};
use crate::publication::patch::data::PatchStreamPosition;
use crate::transactions::data::outcomes::commit_semantic_result::RecoveredRelationalSemanticResult;
use std::sync::Arc;

/// The actual resident Native owner artifact. This cannot qualify a directory
/// Home, mint a performed effect, or reconstruct a live CommitResult.
#[derive(Debug)]
pub struct ObservedRelationalMemoryCommit {
    origin: MemoryCommitOrigin,
    semantic: RecoveredRelationalSemanticResult,
    digest: [u8; 32],
}
#[derive(Debug)]
enum MemoryCommitOrigin {
    Live {
        position: PatchStreamPosition,
        canonical: SharedCanonicalCommitEnvelope,
    },
}

impl ObservedRelationalMemoryCommit {
    pub fn receipt(&self) -> &RelationalCommitReceipt {
        self.semantic.semantic().receipt()
    }
    pub fn semantic_result(&self) -> &RecoveredRelationalSemanticResult {
        &self.semantic
    }
    pub fn position(&self) -> PatchStreamPosition {
        match &self.origin {
            MemoryCommitOrigin::Live { position, .. } => *position,
        }
    }
    pub fn artifact_digest(&self) -> &[u8; 32] {
        &self.digest
    }

    pub(crate) fn matches_original_envelope(
        &self,
        expected: &std::sync::Weak<crate::history::data::CanonicalCommitEnvelope>,
    ) -> bool {
        match &self.origin {
            MemoryCommitOrigin::Live { canonical, .. } => {
                std::ptr::eq(expected.as_ptr(), canonical.as_ref())
            }
        }
    }

    pub fn created_entity(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        entity: crate::identity::data::EntityId,
    ) -> Result<Option<ReadmittedRelationalCreatedEntity<'_>>, Refusal> {
        read_created_entity(
            execution,
            self.semantic.semantic(),
            match &self.origin {
                MemoryCommitOrigin::Live { canonical, .. } => {
                    &canonical.patch.authoritative_record_patches
                }
            },
            entity,
        )
    }
}

/// Operations used by the Relational owners.
pub trait ObservedRelationalMemoryCommitOwnerPort: Sized {
    fn from_live(positioned: PositionedCanonicalCommit, digest: [u8; 32]) -> Self;
}

impl ObservedRelationalMemoryCommitOwnerPort for ObservedRelationalMemoryCommit {
    fn from_live(positioned: PositionedCanonicalCommit, digest: [u8; 32]) -> Self {
        let semantic = assemble_recovered_relational_semantic_result(
            positioned.envelope().semantic_result_arc(),
        );
        let position = positioned.position();
        let canonical =
            SharedCanonicalCommitEnvelope::from_arc(Arc::clone(positioned.canonical_arc()));
        Self {
            semantic,
            origin: MemoryCommitOrigin::Live {
                position,
                canonical,
            },
            digest,
        }
    }
}
