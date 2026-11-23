//! Evidence retained from the exact memory-owned canonical commit.
use super::created_row_read::{read_created_entity, ReadmittedRelationalCreatedEntity};
use super::refusal::RelationalAssociatedEffectRefusal as Refusal;
use crate::durability::canonical_commit_artifact::decoded_quote::CheckpointEffectData;
use crate::durability::checkpoint_admission::admitted_commit::CheckpointAdmittedCommit;
use crate::history::data::{PositionedCanonicalCommit, RelationalCommitReceipt};
use crate::runtime::state::subsystems::services::sequence::origin::RelationalRuntimeOrigin;
use crate::transactions::data::outcomes::commit_semantic_result::RecoveredRelationalSemanticResult;

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
    Live(PositionedCanonicalCommit),
    Interim {
        owner: RelationalRuntimeOrigin,
        admitted: CheckpointAdmittedCommit,
        data: CheckpointEffectData,
    },
}

impl ObservedRelationalMemoryCommit {
    pub(super) fn from_live(positioned: PositionedCanonicalCommit, digest: [u8; 32]) -> Self {
        Self {
            semantic: RecoveredRelationalSemanticResult {
                semantic: positioned.envelope().semantic_result_arc(),
            },
            origin: MemoryCommitOrigin::Live(positioned),
            digest,
        }
    }

    pub(super) fn from_interim(
        owner: RelationalRuntimeOrigin,
        admitted: CheckpointAdmittedCommit,
        data: CheckpointEffectData,
        digest: [u8; 32],
    ) -> Self {
        Self {
            semantic: RecoveredRelationalSemanticResult {
                semantic: data.semantic.clone(),
            },
            origin: MemoryCommitOrigin::Interim {
                owner,
                admitted,
                data,
            },
            digest,
        }
    }

    pub fn receipt(&self) -> &RelationalCommitReceipt {
        self.semantic.semantic().receipt()
    }
    pub fn semantic_result(&self) -> &RecoveredRelationalSemanticResult {
        &self.semantic
    }
    pub fn position(&self) -> crate::publication::patch::data::PatchStreamPosition {
        match &self.origin {
            MemoryCommitOrigin::Live(positioned) => positioned.position(),
            MemoryCommitOrigin::Interim { admitted, .. } => admitted.index().position,
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
            MemoryCommitOrigin::Live(positioned) => std::sync::Weak::ptr_eq(
                expected,
                &std::sync::Arc::downgrade(positioned.canonical_arc()),
            ),
            MemoryCommitOrigin::Interim { .. } => false,
        }
    }

    pub(crate) fn matches_interim(
        &self,
        expected: &RelationalRuntimeOrigin,
        row: &CheckpointAdmittedCommit,
    ) -> bool {
        match &self.origin {
            MemoryCommitOrigin::Live(_) => false,
            MemoryCommitOrigin::Interim {
                owner, admitted, ..
            } => owner.same_origin(expected) && admitted.same_owner(row),
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
                MemoryCommitOrigin::Live(positioned) => {
                    &positioned.envelope().patch.authoritative_record_patches
                }
                MemoryCommitOrigin::Interim { data, .. } => &data.creations,
            },
            entity,
        )
    }
}
