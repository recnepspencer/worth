//! Read-only original effect facts; none of these descriptions grants adoption.
use crate::durability::data::RelationalRecoveryStop;
use crate::durability::medium::RelationalDurableColdSource;
use crate::history::commit::RelationalCommitEnvelopeResidency;
use crate::history::data::RelationalCallerAssociation;
use crate::transactions::data::outcomes::commit_semantic_result::RecoveredRelationalSemanticResult;
use std::sync::{Arc, Weak};

mod created_row;
pub(crate) use worth_relational_branch::internal_port::durability::authority::associated_effects::created_row_read;
pub(super) mod discovery;
mod readmission;
pub use created_row_read::ReadmittedRelationalCreatedEntity;
mod memory_original;
pub(crate) use worth_relational_branch::internal_port::durability::authority::associated_effects::observed_memory_commit;
pub(crate) use worth_relational_contracts::internal_port::durability::authority::associated_effects::refusal;
pub use observed_memory_commit::ObservedRelationalMemoryCommit;
pub use refusal::RelationalAssociatedEffectRefusal;

/// Exact discovered index and weak owner/source identities. The actual Native
/// owner retains source custody; an observer cannot keep a closed Home serving.
/// This is not serializable or a publication capability.
///
/// ```compile_fail,E0277
/// use worth_relational::facade::durability::RelationalAssociatedEffectSubject;
/// fn cannot_readmit_a_source_subject_from_data(bytes: &[u8]) {
///     let _: RelationalAssociatedEffectSubject = rmp_serde::from_slice(bytes).unwrap();
/// }
/// ```
#[derive(Debug, Clone)]
pub struct RelationalAssociatedEffectSubject {
    runtime_instance_id: u64,
    entry: crate::history::commit::RetainedCanonicalIndex,
    source: Weak<dyn RelationalDurableColdSource>,
    residency: Weak<RelationalCommitEnvelopeResidency>,
}

impl RelationalAssociatedEffectSubject {
    pub(super) fn source_entry(&self) -> &crate::history::commit::RelationalCommitIndexEntry {
        &self.entry
    }

    pub fn receipt(&self) -> &crate::history::data::RelationalCommitReceipt {
        &self.entry.commit
    }
    pub fn position(&self) -> crate::publication::patch::data::PatchStreamPosition {
        self.entry.position
    }
    pub fn artifact_digest(&self) -> &[u8; 32] {
        &self.entry.payload_digest
    }
    pub fn roots(&self) -> &crate::branch::target::RelationalBranchRootDescriptor {
        &self.entry.root
    }
    pub fn association(&self) -> Option<&RelationalCallerAssociation> {
        self.entry.association_discovery.association()
    }
}

/// These observations describe Native owner state, never permission to move a
/// product head. In particular checkpoint readmission does not mean settled.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelationalAssociatedEffectSettlement {
    PerformedPending,
    PerformedSettled,
    CheckpointReadmitted,
}

#[derive(Debug)]
pub struct ReadmittedRelationalAssociatedEffect {
    subject: RelationalAssociatedEffectSubject,
    semantic: RecoveredRelationalSemanticResult,
    origin: ReadmittedEffectOrigin,
    settlement: RelationalAssociatedEffectSettlement,
}

#[derive(Debug)]
pub(super) enum ReadmittedEffectOrigin {
    Original(crate::history::data::PositionedCanonicalCommit),
    Checkpoint {
        commit: crate::durability::checkpoint_admission::CheckpointAdmittedCommit,
        data: crate::durability::canonical_commit_artifact::decoded_quote::checkpoint_effect_data::CheckpointEffectData,
    },
}

impl ReadmittedRelationalAssociatedEffect {
    pub(super) fn branch(&self) -> &crate::history::data::BranchId {
        match &self.origin {
            ReadmittedEffectOrigin::Original(positioned) => &positioned.envelope().branch_context,
            ReadmittedEffectOrigin::Checkpoint { commit, .. } => &commit.index().commit.branch_id,
        }
    }
    pub(super) fn origin(&self) -> &ReadmittedEffectOrigin {
        &self.origin
    }
    pub fn subject(&self) -> &RelationalAssociatedEffectSubject {
        &self.subject
    }
    pub fn semantic_result(
        &self,
    ) -> &crate::transactions::data::outcomes::commit_semantic_result::RelationalCommitSemanticResult
    {
        self.semantic.semantic()
    }
    pub fn recovered_semantic_result(&self) -> &RecoveredRelationalSemanticResult {
        &self.semantic
    }
    pub fn settlement(&self) -> RelationalAssociatedEffectSettlement {
        self.settlement
    }
}

impl RelationalAssociatedEffectSubject {
    pub(super) fn belongs_to_runtime(&self, runtime_instance: u64) -> bool {
        self.runtime_instance_id == runtime_instance
    }

    fn bound_residency(&self) -> Option<Arc<RelationalCommitEnvelopeResidency>> {
        let residency = self.residency.upgrade()?;
        let source = self.source.upgrade()?;
        let (entry, actual_source) = residency.source_placement()?;
        if !entry.same_owner(&self.entry) || !Arc::ptr_eq(&actual_source, &source) {
            return None;
        }
        Some(residency)
    }
}
