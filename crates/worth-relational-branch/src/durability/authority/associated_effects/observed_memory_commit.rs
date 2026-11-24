//! Evidence retained from the exact memory-owned canonical commit.
use crate::internal_port::history::data::canonical_commit_envelope::CanonicalCommitEnvelopeOwnerPort as _;
use worth_relational_contracts::internal_port::transactions::data::outcomes::commit_semantic_result::assemble_recovered_relational_semantic_result;
use super::created_row_read::{read_created_entity, ReadmittedRelationalCreatedEntity};
use super::refusal::RelationalAssociatedEffectRefusal as Refusal;
use crate::durability::canonical_commit_artifact::decoded_quote::checkpoint_effect_data::CheckpointEffectData;
use crate::durability::checkpoint_admission::admitted_commit::CheckpointAdmittedCommit;
use crate::history::data::canonical_commit_envelope::shared::SharedCanonicalCommitEnvelopeOwnerPort as _;
use crate::history::data::{
    PositionedCanonicalCommit, RelationalCommitReceipt, SharedCanonicalCommitEnvelope,
};
use crate::publication::patch::data::PatchStreamPosition;
use crate::runtime::state::subsystems::services::sequence::origin::RelationalRuntimeOrigin;
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
    Interim {
        owner: RelationalRuntimeOrigin,
        admitted: CheckpointAdmittedCommit,
        data: CheckpointEffectData,
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
            MemoryCommitOrigin::Live { canonical, .. } => {
                std::ptr::eq(expected.as_ptr(), canonical.as_ref())
            }
            MemoryCommitOrigin::Interim { .. } => false,
        }
    }

    pub(crate) fn matches_interim(
        &self,
        expected: &RelationalRuntimeOrigin,
        row: &CheckpointAdmittedCommit,
    ) -> bool {
        match &self.origin {
            MemoryCommitOrigin::Live { .. } => false,
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
                MemoryCommitOrigin::Live { canonical, .. } => {
                    &canonical.patch.authoritative_record_patches
                }
                MemoryCommitOrigin::Interim { data, .. } => &data.creations,
            },
            entity,
        )
    }
}

/// Operations used by the Relational owners.
pub trait ObservedRelationalMemoryCommitOwnerPort: Sized {
    fn from_live(positioned: PositionedCanonicalCommit, digest: [u8; 32]) -> Self;
    fn from_interim(
        owner: RelationalRuntimeOrigin,
        admitted: CheckpointAdmittedCommit,
        data: CheckpointEffectData,
        digest: [u8; 32],
    ) -> Self;
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

    fn from_interim(
        owner: RelationalRuntimeOrigin,
        admitted: CheckpointAdmittedCommit,
        data: CheckpointEffectData,
        digest: [u8; 32],
    ) -> Self {
        Self {
            semantic: assemble_recovered_relational_semantic_result(data.semantic.clone()),
            origin: MemoryCommitOrigin::Interim {
                owner,
                admitted,
                data,
            },
            digest,
        }
    }
}
