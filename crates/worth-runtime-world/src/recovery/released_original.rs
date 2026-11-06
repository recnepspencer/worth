//! Terminal original meaning owns no live result, basis, recovery claim or service.
use crate::durability::AdmittedWorldDisposition;
use crate::identity::{
    CompositeCommitIdentity, CompositePublicationAttemptIdentity, ProductBranchIdentity,
    ProductBranchIncarnation, ProductUnpublishedOwnerEffectsIdentity,
};
use std::sync::{Arc, OnceLock};
use worth_execution::ExecutionMemoryReservation;
use worth_foundational::facade::{
    CanonicalDigestId, CompositeAttemptAssociation, RecordedProductBranchReference,
};
use worth_relational::facade::{
    durability::{
        ObservedRelationalMemoryCommit, ReadmittedRelationalAssociatedEffect,
        ReadmittedRelationalCreatedEntity, RelationalAssociatedEffectRefusal,
    },
    history::RelationalCommitReceipt,
    identity::EntityId,
    publication::PatchStreamPosition,
    transactions::RelationalCommitSemanticResult,
};

#[derive(Debug)]
pub(super) enum ReleasedOriginalSource {
    Directory {
        disposition: AdmittedWorldDisposition,
        native: Option<ReadmittedRelationalAssociatedEffect>,
    },
    Memory {
        native: Option<ObservedRelationalMemoryCommit>,
    },
}
#[derive(Debug)]
pub(super) struct ReleasedOriginalSlot {
    pub(super) identity: ProductUnpublishedOwnerEffectsIdentity,
    pub(super) attempt: CompositePublicationAttemptIdentity,
    pub(super) commit: Option<CompositeCommitIdentity>,
    pub(super) branch: ProductBranchIdentity,
    pub(super) incarnation: ProductBranchIncarnation,
    pub(super) expected: RecordedProductBranchReference,
    pub(super) source: OnceLock<ReleasedOriginalSource>,
    pub(super) _funding: Option<ExecutionMemoryReservation>,
}

/// Read-only original effect and actual release. This is neither an application
/// commit receipt nor authority to adopt, continue or release owner custody.
/// Memory origin has no durable Open, Home binding or World journal location.
///
/// ```compile_fail,E0451
/// use worth_runtime_world::facade::WorldReleasedOriginal;
/// let _ = WorldReleasedOriginal { slot: panic!() };
/// ```
#[derive(Clone, Debug)]
pub struct WorldReleasedOriginal {
    slot: Arc<ReleasedOriginalSlot>,
}
impl WorldReleasedOriginal {
    pub(super) fn from_sealed(slot: Arc<ReleasedOriginalSlot>) -> Self {
        assert!(
            slot.source.get().is_some(),
            "only terminal source is visible"
        );
        Self { slot }
    }
    fn source(&self) -> &ReleasedOriginalSource {
        self.slot.source.get().expect("sealed terminal source")
    }
    pub fn identity(&self) -> &ProductUnpublishedOwnerEffectsIdentity {
        &self.slot.identity
    }
    pub fn attempt_identity(&self) -> &CompositePublicationAttemptIdentity {
        &self.slot.attempt
    }
    pub fn reserved_commit_identity(&self) -> Option<&CompositeCommitIdentity> {
        self.slot.commit.as_ref()
    }
    pub fn product_branch(&self) -> &ProductBranchIdentity {
        &self.slot.branch
    }
    pub fn product_incarnation(&self) -> ProductBranchIncarnation {
        self.slot.incarnation
    }
    pub fn expected_product(&self) -> &RecordedProductBranchReference {
        &self.slot.expected
    }
    pub fn original_association(&self) -> Option<&CompositeAttemptAssociation> {
        match self.source() {
            ReleasedOriginalSource::Directory { disposition, .. } => {
                Some(&disposition.open().association)
            }
            ReleasedOriginalSource::Memory { .. } => None,
        }
    }
    pub fn release_journal_position(&self) -> Option<u64> {
        match self.source() {
            ReleasedOriginalSource::Directory { disposition, .. } => Some(disposition.position()),
            ReleasedOriginalSource::Memory { .. } => None,
        }
    }
    pub fn release_artifact_digest(&self) -> Option<CanonicalDigestId> {
        match self.source() {
            ReleasedOriginalSource::Directory { disposition, .. } => {
                Some(CanonicalDigestId::new(*disposition.digest()))
            }
            ReleasedOriginalSource::Memory { .. } => None,
        }
    }
    pub fn native_semantic_result(&self) -> Option<&RelationalCommitSemanticResult> {
        match self.source() {
            ReleasedOriginalSource::Directory { native, .. } => {
                native.as_ref().map(|value| value.semantic_result())
            }
            ReleasedOriginalSource::Memory { native } => native
                .as_ref()
                .map(|value| value.semantic_result().semantic()),
        }
    }
    pub fn native_receipt(&self) -> Option<&RelationalCommitReceipt> {
        self.native_semantic_result().map(|value| value.receipt())
    }
    pub fn native_position(&self) -> Option<PatchStreamPosition> {
        match self.source() {
            ReleasedOriginalSource::Directory { native, .. } => {
                native.as_ref().map(|value| value.subject().position())
            }
            ReleasedOriginalSource::Memory { native } => {
                native.as_ref().map(|value| value.position())
            }
        }
    }
    pub fn native_artifact_digest(&self) -> Option<CanonicalDigestId> {
        match self.source() {
            ReleasedOriginalSource::Directory { native, .. } => native
                .as_ref()
                .map(|value| CanonicalDigestId::new(*value.subject().artifact_digest())),
            ReleasedOriginalSource::Memory { native } => native
                .as_ref()
                .map(|value| CanonicalDigestId::new(*value.artifact_digest())),
        }
    }
    pub fn created_entity(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        identity: EntityId,
    ) -> Result<Option<ReadmittedRelationalCreatedEntity<'_>>, RelationalAssociatedEffectRefusal>
    {
        match self.source() {
            ReleasedOriginalSource::Directory { native, .. } => match native {
                Some(native) => native.created_entity(execution, identity),
                None => Ok(None),
            },
            ReleasedOriginalSource::Memory { native } => match native {
                Some(native) => native.created_entity(execution, identity),
                None => Ok(None),
            },
        }
    }
    pub fn same_original(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.slot, &other.slot)
    }
}
impl PartialEq for WorldReleasedOriginal {
    fn eq(&self, other: &Self) -> bool {
        self.slot.identity == other.slot.identity
            && self.slot.attempt == other.slot.attempt
            && self.slot.commit == other.slot.commit
            && self.slot.expected == other.slot.expected
            && self.original_association() == other.original_association()
            && self.release_journal_position() == other.release_journal_position()
            && self.release_artifact_digest() == other.release_artifact_digest()
            && self.native_position() == other.native_position()
            && self.native_artifact_digest() == other.native_artifact_digest()
            && self.native_semantic_result() == other.native_semantic_result()
    }
}
impl Eq for WorldReleasedOriginal {}

/// Bounded terminal source observations, excluded from active custody counts.
#[derive(Debug)]
pub struct WorldReleasedOriginalPage {
    pub(super) rows: Vec<WorldReleasedOriginal>,
    pub(super) next_after: Option<ProductUnpublishedOwnerEffectsIdentity>,
    pub(super) _funding: Option<ExecutionMemoryReservation>,
}
impl WorldReleasedOriginalPage {
    pub fn rows(&self) -> &[WorldReleasedOriginal] {
        &self.rows
    }
    pub fn next_after(&self) -> Option<&ProductUnpublishedOwnerEffectsIdentity> {
        self.next_after.as_ref()
    }
}
