//! Read-only original effect facts; none of these descriptions grants adoption.
use contracts::transactions::data::outcomes::commit_semantic_result::RecoveredRelationalSemanticResult;
use std::any::Any;
use std::sync::{Arc, Weak};
use worth_relational_branch::internal_port::history::commit::envelope_residency::RelationalCommitEnvelopeResidency;
use worth_relational_branch::internal_port::history::data::caller_association::RelationalCallerAssociation;
use worth_relational_branch::internal_port::history::data::SharedCanonicalCommitEnvelope;
use worth_relational_contracts::internal_port as contracts;
use worth_relational_storage::internal_port::durability::medium::cold_source::RelationalDurableColdSource;

trait EnvelopeResidencyBacking: std::fmt::Debug + Send + Sync {
    fn into_any(self: Arc<Self>) -> Arc<dyn Any + Send + Sync>;
}

impl EnvelopeResidencyBacking for RelationalCommitEnvelopeResidency {
    fn into_any(self: Arc<Self>) -> Arc<dyn Any + Send + Sync> {
        self
    }
}

pub mod created_row;
pub use worth_relational_branch::internal_port::durability::authority::associated_effects::{
    created_row_read, observed_memory_commit,
};

pub use worth_relational_branch::internal_port::durability::authority::associated_effects::created_row_read::ReadmittedRelationalCreatedEntity;

pub use contracts::durability::authority::associated_effects::refusal;
pub use worth_relational_branch::internal_port::durability::authority::associated_effects::observed_memory_commit::ObservedRelationalMemoryCommit;
pub use contracts::durability::authority::associated_effects::refusal::RelationalAssociatedEffectRefusal;

/// Exact discovered index and weak owner/source identities. The actual Native
/// owner retains source custody; an observer cannot keep a closed Home serving.
/// This is not serializable or a publication capability.
#[derive(Debug, Clone)]
pub struct RelationalAssociatedEffectSubject {
    runtime_instance_id: u64,
    entry: worth_relational_branch::internal_port::history::commit::retained_index::RetainedCanonicalIndex,
    source: Weak<dyn RelationalDurableColdSource>,
    residency: Weak<dyn EnvelopeResidencyBacking>,
}

impl RelationalAssociatedEffectSubject {
    pub(super) fn source_entry(&self) -> &worth_relational_branch::internal_port::history::commit::index_entry::RelationalCommitIndexEntry{
        &self.entry
    }

    pub fn receipt(&self) -> &contracts::history::data::commit_receipt::RelationalCommitReceipt {
        &self.entry.commit
    }
    pub fn position(
        &self,
    ) -> contracts::publication::patch::data::patch_position::PatchStreamPosition {
        self.entry.position
    }
    pub fn artifact_digest(&self) -> &[u8; 32] {
        &self.entry.payload_digest
    }
    pub fn roots(&self) -> &contracts::branch::target::RelationalBranchRootDescriptor {
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
pub enum ReadmittedEffectOrigin {
    Original(SharedCanonicalCommitEnvelope),
    Checkpoint {
        commit: worth_relational_branch::internal_port::durability::checkpoint_admission::admitted_commit::CheckpointAdmittedCommit,
        data: worth_relational_branch::internal_port::durability::canonical_commit_artifact::decoded_quote::checkpoint_effect_data::CheckpointEffectData,
    },
}

impl ReadmittedRelationalAssociatedEffect {
    pub(super) fn branch(&self) -> &contracts::history::data::coordinates::BranchId {
        match &self.origin {
            ReadmittedEffectOrigin::Original(envelope) => &envelope.branch_context,
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
    ) -> &contracts::transactions::data::outcomes::commit_semantic_result::RelationalCommitSemanticResult
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
        let residency = self
            .residency
            .upgrade()?
            .into_any()
            .downcast::<RelationalCommitEnvelopeResidency>()
            .expect(
                "RelationalAssociatedEffectSubject always wraps RelationalCommitEnvelopeResidency",
            );
        let source = self.source.upgrade()?;
        let (entry, actual_source) = residency.source_placement()?;
        if !entry.same_owner(&self.entry) || !Arc::ptr_eq(&actual_source, &source) {
            return None;
        }
        Some(residency)
    }
}

#[doc(hidden)]
pub trait RelationalAssociatedEffectSubjectOwnerPort: Sized {
    fn retained_entry(&self) -> &worth_relational_branch::internal_port::history::commit::retained_index::RetainedCanonicalIndex;
    fn bound_source(&self) -> Option<Arc<dyn RelationalDurableColdSource>>;
    fn capture_subject(
        runtime_instance_id: u64,
        entry: worth_relational_branch::internal_port::history::commit::retained_index::RetainedCanonicalIndex,
        source: Weak<dyn RelationalDurableColdSource>,
        residency: Weak<RelationalCommitEnvelopeResidency>,
    ) -> Self;

    fn source_entry(&self) -> &worth_relational_branch::internal_port::history::commit::index_entry::RelationalCommitIndexEntry;

    fn belongs_to_runtime(&self, runtime_instance: u64) -> bool;

    fn bound_residency(&self) -> Option<Arc<RelationalCommitEnvelopeResidency>>;
}

impl RelationalAssociatedEffectSubjectOwnerPort for RelationalAssociatedEffectSubject {
    fn retained_entry(&self) -> &worth_relational_branch::internal_port::history::commit::retained_index::RetainedCanonicalIndex{
        &self.entry
    }

    fn bound_source(&self) -> Option<Arc<dyn RelationalDurableColdSource>> {
        self.source.upgrade()
    }

    fn capture_subject(
        runtime_instance_id: u64,
        entry: worth_relational_branch::internal_port::history::commit::retained_index::RetainedCanonicalIndex,
        source: Weak<dyn RelationalDurableColdSource>,
        residency: Weak<RelationalCommitEnvelopeResidency>,
    ) -> Self {
        Self {
            runtime_instance_id,
            entry,
            source,
            residency,
        }
    }

    fn source_entry(&self) -> &worth_relational_branch::internal_port::history::commit::index_entry::RelationalCommitIndexEntry{
        self.source_entry()
    }

    fn belongs_to_runtime(&self, runtime_instance: u64) -> bool {
        self.belongs_to_runtime(runtime_instance)
    }

    fn bound_residency(&self) -> Option<Arc<RelationalCommitEnvelopeResidency>> {
        self.bound_residency()
    }
}

#[doc(hidden)]
pub trait ReadmittedRelationalAssociatedEffectOwnerPort: Sized {
    fn from_readmission(
        subject: RelationalAssociatedEffectSubject,
        semantic: RecoveredRelationalSemanticResult,
        origin: ReadmittedEffectOrigin,
        settlement: RelationalAssociatedEffectSettlement,
    ) -> Self;

    fn branch(&self) -> &contracts::history::data::coordinates::BranchId;
}

impl ReadmittedRelationalAssociatedEffectOwnerPort for ReadmittedRelationalAssociatedEffect {
    fn from_readmission(
        subject: RelationalAssociatedEffectSubject,
        semantic: RecoveredRelationalSemanticResult,
        origin: ReadmittedEffectOrigin,
        settlement: RelationalAssociatedEffectSettlement,
    ) -> Self {
        Self {
            subject,
            semantic,
            origin,
            settlement,
        }
    }

    fn branch(&self) -> &contracts::history::data::coordinates::BranchId {
        self.branch()
    }
}
