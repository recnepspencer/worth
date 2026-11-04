//! Immutable recovered meaning owns the original Native admission exactly once.
mod retirement;
use crate::{
    branch::ProductBranchReferenceSnapshot, identity::CompositePublicationAttemptIdentity,
};
pub(crate) use retirement::ReadmittedPublicationRetirement;
use std::sync::{Arc, OnceLock};
use worth_proof::RelationalPublicationReadmitted;
use worth_relational::facade::{
    durability::RelationalRecoveredPublicationCustody, transactions::RelationalCommitSemanticResult,
};

#[derive(Debug)]
pub(crate) struct ReadmittedPublicationRecord {
    pub(crate) before: ProductBranchReferenceSnapshot,
    pub(crate) after: ProductBranchReferenceSnapshot,
    pub(crate) attempt: CompositePublicationAttemptIdentity,
    location: OnceLock<crate::durability::AdmittedWorldOccurrenceLocation>,
    pub(crate) incarnation: worth_foundational::facade::HomeIncarnation,
    native: Option<NativePublicationMeaning>,
}
#[derive(Debug)]
pub(super) enum NativePublicationMeaning {
    Recovered(RelationalRecoveredPublicationCustody),
    Source(worth_relational::facade::durability::ReadmittedRelationalAssociatedEffect),
    Retiring(worth_relational::facade::durability::RelationalRecoveredPublicationRetirement),
}
impl NativePublicationMeaning {
    fn effect(
        &self,
    ) -> &worth_relational::facade::durability::ReadmittedRelationalAssociatedEffect {
        match self {
            Self::Recovered(custody) => custody.effect(),
            Self::Source(effect) => effect,
            Self::Retiring(_) => unreachable!("only serving records expose Native meaning"),
        }
    }
}
/// This unpublished phase exposes Native meaning, never serving history.
#[derive(Debug)]
pub(crate) struct PreparedReadmittedPublicationRecord {
    record: Arc<ReadmittedPublicationRecord>,
}
impl PreparedReadmittedPublicationRecord {
    pub(crate) fn effect(
        &self,
    ) -> &worth_relational::facade::durability::ReadmittedRelationalAssociatedEffect {
        self.record.effect()
    }
    pub(crate) fn into_retirement(self: Arc<Self>) -> ReadmittedPublicationRetirement {
        let original =
            Arc::try_unwrap(self).expect("sealed close exclusively owns the prepared source view");
        ReadmittedPublicationRetirement::new(original.record)
    }
    pub(crate) fn seal(
        &self,
        occurrence: &crate::durability::AdmittedWorldOccurrence,
    ) -> Arc<ReadmittedPublicationRecord> {
        self.record.seal_occurrence(occurrence);
        self.record.clone()
    }
}
impl ReadmittedPublicationRecord {
    pub(crate) fn prepare_original(
        before: ProductBranchReferenceSnapshot,
        after: ProductBranchReferenceSnapshot,
        attempt: CompositePublicationAttemptIdentity,
        incarnation: worth_foundational::facade::HomeIncarnation,
        native: RelationalPublicationReadmitted<RelationalRecoveredPublicationCustody>,
    ) -> Arc<PreparedReadmittedPublicationRecord> {
        Arc::new(PreparedReadmittedPublicationRecord {
            record: Arc::new(Self {
                before,
                after,
                attempt,
                incarnation,
                location: OnceLock::new(),
                native: Some(NativePublicationMeaning::Recovered(
                    native.consume(|native| native),
                )),
            }),
        })
    }
    pub(crate) fn from_source(
        before: ProductBranchReferenceSnapshot,
        after: ProductBranchReferenceSnapshot,
        attempt: CompositePublicationAttemptIdentity,
        occurrence: &crate::durability::AdmittedWorldOccurrence,
        native: worth_relational::facade::durability::ReadmittedRelationalAssociatedEffect,
    ) -> Arc<Self> {
        Arc::new(Self {
            before,
            after,
            attempt,
            location: OnceLock::from(occurrence.location().clone()),
            incarnation: occurrence.publishing_incarnation(),
            native: Some(NativePublicationMeaning::Source(native)),
        })
    }
    pub(crate) fn location(&self) -> &crate::durability::AdmittedWorldOccurrenceLocation {
        self.location
            .get()
            .expect("only sealed records enter serving history")
    }
    pub(crate) fn effect(
        &self,
    ) -> &worth_relational::facade::durability::ReadmittedRelationalAssociatedEffect {
        self.native().effect()
    }
    fn seal_occurrence(&self, occurrence: &crate::durability::AdmittedWorldOccurrence) {
        assert_eq!(
            self.after.selected_commit().ordinal(),
            occurrence.open().commit.ordinal()
        );
        self.location
            .set(occurrence.location().clone())
            .expect("one actual occurrence location");
    }
    fn native(&self) -> &NativePublicationMeaning {
        self.native
            .as_ref()
            .expect("only serving record exposes Native meaning")
    }
    pub(crate) fn semantic(&self) -> &RelationalCommitSemanticResult {
        self.native().effect().semantic_result()
    }
}

/// One catalog entry carries either live delivery or owner-admitted recovered
/// meaning. Recovered meaning cannot satisfy a live delivery/Performed door.
#[derive(Debug)]
pub(crate) enum PublicationEvidence {
    Live(Arc<super::CanonicalPublicationEnvelope>),
    Readmitted(Arc<ReadmittedPublicationRecord>),
}
impl PublicationEvidence {
    pub(crate) fn live(&self) -> Option<&Arc<super::CanonicalPublicationEnvelope>> {
        match self {
            Self::Live(live) => Some(live),
            Self::Readmitted(_) => None,
        }
    }
    pub(crate) fn facts(&self) -> Option<&super::PerformedPublicationFacts> {
        self.live()?.facts()
    }
    pub(crate) fn awaits_delivery(&self) -> bool {
        self.live().is_some_and(|live| live.awaits_delivery())
    }
}

#[derive(Debug)]
pub enum WorldReadmittedPublicationDenial {
    Request(worth_execution::WorkCeilingDenial),
    ForeignHome,
    OwnerUnavailable,
    History(crate::history::CompositeHistoryCatalogDenial),
    NotReadmitted,
    Source(Box<crate::durability::WorldReadmissionDenial>),
    NativeMismatch,
}
/// An owning read of original recovered meaning. Its exact history protection
/// and catalog keep the SAME record and Native leases alive across head moves.
/// There is no live result, snapshot, timing or publication method.
///
/// ```compile_fail,E0308
/// use worth_runtime_world::facade::{WorldReadmittedPublication, PerformedCompositePublication};
/// fn live_delivery(value: WorldReadmittedPublication) -> PerformedCompositePublication { value }
/// ```
///
/// ```compile_fail,E0599
/// use worth_runtime_world::facade::WorldReadmittedPublication;
/// fn live_result(value: WorldReadmittedPublication) { value.into_live_result(); }
/// ```
///
/// ```
/// use worth_runtime_world::facade::WorldReadmittedPublication;
/// fn original_meaning(value: &WorldReadmittedPublication) {
///     let _ = (value.before(), value.after(), value.native_receipt(), value.semantic_result());
/// }
/// ```
#[derive(Debug)]
pub struct WorldReadmittedPublication {
    pub(crate) record: Arc<ReadmittedPublicationRecord>,
    pub(in crate::history) _history:
        crate::history::retention::CompositeHistoryProtectionObligation,
    pub(crate) _catalog: crate::history::CompositeHistoryCatalog,
}
impl WorldReadmittedPublication {
    pub fn created_entity(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        entity: worth_relational::facade::identity::EntityId,
    ) -> Result<
        Option<worth_relational::facade::durability::ReadmittedRelationalCreatedEntity<'_>>,
        worth_relational::facade::durability::RelationalAssociatedEffectRefusal,
    > {
        self.record
            .native()
            .effect()
            .created_entity(execution, entity)
    }
    pub fn before(&self) -> &ProductBranchReferenceSnapshot {
        &self.record.before
    }
    pub fn after(&self) -> &ProductBranchReferenceSnapshot {
        &self.record.after
    }
    pub fn attempt_identity(&self) -> &CompositePublicationAttemptIdentity {
        &self.record.attempt
    }
    pub fn semantic_result(&self) -> &RelationalCommitSemanticResult {
        self.record.semantic()
    }
    pub fn native_receipt(&self) -> &worth_relational::facade::history::RelationalCommitReceipt {
        self.semantic_result().receipt()
    }
    pub fn journal_position(&self) -> u64 {
        self.record.location().position()
    }
    pub fn artifact_digest(&self) -> &[u8; 32] {
        self.record.location().digest()
    }
    pub fn native_patch_position(
        &self,
    ) -> worth_relational::facade::publication::PatchStreamPosition {
        self.record.native().effect().subject().position()
    }
    pub fn native_artifact_digest(&self) -> &[u8; 32] {
        self.record.native().effect().subject().artifact_digest()
    }
    pub fn admitted_incarnation(&self) -> worth_foundational::facade::HomeIncarnation {
        self.record.incarnation
    }
    pub fn recovered_semantic_result(
        &self,
    ) -> &worth_relational::facade::durability::RecoveredRelationalSemanticResult {
        self.record.native().effect().recovered_semantic_result()
    }
}
