//! Root-free original durable Home meaning, independent of live history retention.
use worth_foundational::facade::{
    DurableCompositePublicationAttemptIdentity, DurableProductBranchReference, HomeIncarnation,
};
use worth_relational::facade::{
    branch::RecordedRelationalComponentReference,
    durability::{
        ReadmittedRelationalAssociatedEffect, ReadmittedRelationalCreatedEntity,
        RecoveredRelationalSemanticResult, RelationalAssociatedEffectRefusal,
    },
    history::RelationalCommitReceipt,
    identity::EntityId,
    publication::PatchStreamPosition,
    transactions::RelationalCommitSemanticResult,
};

/// A request-local read of one durable Home's original World occurrence and
/// Native effect. Unlike [`crate::history::WorldReadmittedPublication`], this
/// holds no live branch basis, root, result, or history-retention pin, so it
/// remains readable after durable history retirement and across reopens.
///
/// ```compile_fail,E0308
/// use worth_runtime_world::facade::{WorldObservedHomePublication, PerformedCompositePublication};
/// fn live_delivery(value: WorldObservedHomePublication) -> PerformedCompositePublication { value }
/// ```
#[derive(Debug)]
pub struct WorldObservedHomePublication {
    before: DurableProductBranchReference,
    after: DurableProductBranchReference,
    product_branch: crate::identity::ProductBranchIdentity,
    product_incarnation: crate::identity::ProductBranchIncarnation,
    product_generation: crate::identity::ProductBranchReferenceGeneration,
    composite_commit: crate::identity::CompositeCommitIdentity,
    publication_attempt: crate::identity::CompositePublicationAttemptIdentity,
    signal_posture: crate::history::CompositeComponentChangePosture,
    signal_source: Option<worth_signal::facade::durability::SignalRecordedSourcePoint>,
    attempt: DurableCompositePublicationAttemptIdentity,
    journal_position: u64,
    artifact_digest: [u8; 32],
    incarnation: HomeIncarnation,
    recorded_before_component: Option<RecordedRelationalComponentReference>,
    native: ReadmittedRelationalAssociatedEffect,
}

impl WorldObservedHomePublication {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn from_verified_occurrence(
        before: DurableProductBranchReference,
        after: DurableProductBranchReference,
        product_branch: crate::identity::ProductBranchIdentity,
        product_incarnation: crate::identity::ProductBranchIncarnation,
        product_generation: crate::identity::ProductBranchReferenceGeneration,
        composite_commit: crate::identity::CompositeCommitIdentity,
        publication_attempt: crate::identity::CompositePublicationAttemptIdentity,
        signal_posture: crate::history::CompositeComponentChangePosture,
        signal_source: Option<worth_signal::facade::durability::SignalRecordedSourcePoint>,
        attempt: DurableCompositePublicationAttemptIdentity,
        journal_position: u64,
        artifact_digest: [u8; 32],
        incarnation: HomeIncarnation,
        recorded_before_component: Option<RecordedRelationalComponentReference>,
        native: ReadmittedRelationalAssociatedEffect,
    ) -> Self {
        Self {
            before,
            after,
            product_branch,
            product_incarnation,
            product_generation,
            composite_commit,
            publication_attempt,
            signal_posture,
            signal_source,
            attempt,
            journal_position,
            artifact_digest,
            incarnation,
            recorded_before_component,
            native,
        }
    }

    pub fn before(&self) -> &DurableProductBranchReference {
        &self.before
    }
    pub fn after(&self) -> &DurableProductBranchReference {
        &self.after
    }
    pub fn product_branch(&self) -> &crate::identity::ProductBranchIdentity {
        &self.product_branch
    }
    pub fn product_incarnation(&self) -> crate::identity::ProductBranchIncarnation {
        self.product_incarnation
    }
    pub fn product_generation(&self) -> crate::identity::ProductBranchReferenceGeneration {
        self.product_generation
    }
    pub fn composite_commit(&self) -> &crate::identity::CompositeCommitIdentity {
        &self.composite_commit
    }
    pub fn publication_attempt(&self) -> &crate::identity::CompositePublicationAttemptIdentity {
        &self.publication_attempt
    }
    pub fn signal_posture(&self) -> crate::history::CompositeComponentChangePosture {
        self.signal_posture
    }
    pub fn signal_source(
        &self,
    ) -> Option<&worth_signal::facade::durability::SignalRecordedSourcePoint> {
        self.signal_source.as_ref()
    }
    pub fn attempt_identity(&self) -> DurableCompositePublicationAttemptIdentity {
        self.attempt
    }
    pub fn journal_position(&self) -> u64 {
        self.journal_position
    }
    pub fn artifact_digest(&self) -> &[u8; 32] {
        &self.artifact_digest
    }
    pub fn admitted_incarnation(&self) -> HomeIncarnation {
        self.incarnation
    }
    pub fn recorded_before_component(&self) -> Option<&RecordedRelationalComponentReference> {
        self.recorded_before_component.as_ref()
    }
    pub fn native(&self) -> &ReadmittedRelationalAssociatedEffect {
        &self.native
    }
    pub fn matches_receiver(&self, observation: &crate::branch::ProductBranchObservation) -> bool {
        observation.owner_identity().durable() == Some(self.after.branch().owner())
            && observation.branch_identity().name().as_str() == self.after.branch().name()
            && observation.lifecycle_incarnation().ordinal() == self.after.incarnation().ordinal()
    }
    pub fn created_entity(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        entity: EntityId,
    ) -> Result<Option<ReadmittedRelationalCreatedEntity<'_>>, RelationalAssociatedEffectRefusal>
    {
        self.native.created_entity(execution, entity)
    }
    pub fn semantic_result(&self) -> &RelationalCommitSemanticResult {
        self.native.semantic_result()
    }
    pub fn native_receipt(&self) -> &RelationalCommitReceipt {
        self.native.semantic_result().receipt()
    }
    pub fn native_patch_position(&self) -> PatchStreamPosition {
        self.native.subject().position()
    }
    pub fn native_artifact_digest(&self) -> &[u8; 32] {
        self.native.subject().artifact_digest()
    }
    pub fn recovered_semantic_result(&self) -> &RecoveredRelationalSemanticResult {
        self.native.recovered_semantic_result()
    }
}
