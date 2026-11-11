//! Immutable recovered meaning retains every original acting owner exactly once.
mod backing;
mod native_view;
mod owner_evidence;
mod retirement;
use crate::{
    branch::ProductBranchReferenceSnapshot, identity::CompositePublicationAttemptIdentity,
};
pub(crate) use backing::ReadmittedPublicationRecordBacking;
pub(crate) use native_view::NativeReadmittedPublicationRecord;
use owner_evidence::{NativePublicationMeaning, ReadmittedOwnerMeaning};
pub(crate) use owner_evidence::{RecoveredWorldOwnerEvidence, RecoveredWorldOwnerEvidenceRef};
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
    owners: Option<ReadmittedOwnerMeaning>,
    _memory: Option<worth_execution::ExecutionMemoryReservation>,
}
/// This unpublished phase exposes Native meaning, never serving history.
#[derive(Debug)]
pub(crate) struct PreparedReadmittedPublicationRecord {
    record: Arc<ReadmittedPublicationRecord>,
    native: Option<Arc<NativePublicationMeaning>>,
}
impl PreparedReadmittedPublicationRecord {
    pub(crate) fn effect(
        &self,
    ) -> Option<&worth_relational::facade::durability::ReadmittedRelationalAssociatedEffect> {
        self.native.as_ref().map(|native| native.effect())
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
        backing: ReadmittedPublicationRecordBacking,
        before: ProductBranchReferenceSnapshot,
        after: ProductBranchReferenceSnapshot,
        attempt: CompositePublicationAttemptIdentity,
        incarnation: worth_foundational::facade::HomeIncarnation,
        owners: RecoveredWorldOwnerEvidence,
    ) -> Arc<PreparedReadmittedPublicationRecord> {
        let owners = owners.into_meaning();
        Arc::new(PreparedReadmittedPublicationRecord {
            native: owners.native().cloned(),
            record: Arc::new(Self {
                before,
                after,
                attempt,
                incarnation,
                location: OnceLock::new(),
                _memory: backing.memory,
                owners: Some(owners),
            }),
        })
    }
    pub(crate) fn new_native(
        prepared: ReadmittedPublicationRecordBacking,
        before: ProductBranchReferenceSnapshot,
        after: ProductBranchReferenceSnapshot,
        attempt: CompositePublicationAttemptIdentity,
        location: crate::durability::AdmittedWorldOccurrenceLocation,
        incarnation: worth_foundational::facade::HomeIncarnation,
        native: RelationalPublicationReadmitted<RelationalRecoveredPublicationCustody>,
    ) -> NativeReadmittedPublicationRecord {
        let native = Arc::new(NativePublicationMeaning::Recovered(
            native.consume(|native| native),
        ));
        let record = Self::from_meaning(
            prepared,
            before,
            after,
            attempt,
            location,
            incarnation,
            ReadmittedOwnerMeaning::RelationalOnly(native.clone()),
        );
        NativeReadmittedPublicationRecord { record, native }
    }
    pub(crate) fn new(
        prepared: ReadmittedPublicationRecordBacking,
        before: ProductBranchReferenceSnapshot,
        after: ProductBranchReferenceSnapshot,
        attempt: CompositePublicationAttemptIdentity,
        location: crate::durability::AdmittedWorldOccurrenceLocation,
        incarnation: worth_foundational::facade::HomeIncarnation,
        owners: RecoveredWorldOwnerEvidence,
    ) -> Arc<Self> {
        Self::from_meaning(
            prepared,
            before,
            after,
            attempt,
            location,
            incarnation,
            owners.into_meaning(),
        )
    }
    pub(crate) fn from_source(
        prepared: ReadmittedPublicationRecordBacking,
        before: ProductBranchReferenceSnapshot,
        after: ProductBranchReferenceSnapshot,
        attempt: CompositePublicationAttemptIdentity,
        occurrence: &crate::durability::AdmittedWorldOccurrence,
        native: worth_relational::facade::durability::ReadmittedRelationalAssociatedEffect,
    ) -> Arc<Self> {
        Self::from_meaning(
            prepared,
            before,
            after,
            attempt,
            occurrence.location().clone(),
            occurrence.publishing_incarnation(),
            ReadmittedOwnerMeaning::RelationalOnly(Arc::new(NativePublicationMeaning::Source(
                native,
            ))),
        )
    }

    fn from_meaning(
        prepared: ReadmittedPublicationRecordBacking,
        before: ProductBranchReferenceSnapshot,
        after: ProductBranchReferenceSnapshot,
        attempt: CompositePublicationAttemptIdentity,
        location: crate::durability::AdmittedWorldOccurrenceLocation,
        incarnation: worth_foundational::facade::HomeIncarnation,
        owners: ReadmittedOwnerMeaning,
    ) -> Arc<Self> {
        Arc::new(Self {
            before,
            after,
            attempt,
            location: OnceLock::from(location),
            incarnation,
            owners: Some(owners),
            _memory: prepared.memory,
        })
    }
    pub(crate) fn location(&self) -> &crate::durability::AdmittedWorldOccurrenceLocation {
        self.location
            .get()
            .expect("only sealed records enter serving history")
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
    pub(crate) fn native_view(self: &Arc<Self>) -> Option<NativeReadmittedPublicationRecord> {
        let native = self.owners.as_ref()?.native()?.clone();
        Some(NativeReadmittedPublicationRecord {
            record: self.clone(),
            native,
        })
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
/// An owning read of original Native-bearing recovered meaning. Its exact history protection
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
    native: Arc<NativePublicationMeaning>,
    pub(in crate::history) _history:
        crate::history::retention::CompositeHistoryProtectionObligation,
    pub(crate) _catalog: crate::history::CompositeHistoryCatalog,
}
impl WorldReadmittedPublication {
    pub(in crate::history) fn from_native(
        original: NativeReadmittedPublicationRecord,
        history: crate::history::retention::CompositeHistoryProtectionObligation,
        catalog: crate::history::CompositeHistoryCatalog,
    ) -> Self {
        Self {
            record: original.record,
            native: original.native,
            _history: history,
            _catalog: catalog,
        }
    }

    pub fn created_entity(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        entity: worth_relational::facade::identity::EntityId,
    ) -> Result<
        Option<worth_relational::facade::durability::ReadmittedRelationalCreatedEntity<'_>>,
        worth_relational::facade::durability::RelationalAssociatedEffectRefusal,
    > {
        self.native.effect().created_entity(execution, entity)
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
        self.native.effect().semantic_result()
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
        self.native.effect().subject().position()
    }
    pub fn native_artifact_digest(&self) -> &[u8; 32] {
        self.native.effect().subject().artifact_digest()
    }
    pub fn admitted_incarnation(&self) -> worth_foundational::facade::HomeIncarnation {
        self.record.incarnation
    }
    pub fn recovered_semantic_result(
        &self,
    ) -> &worth_relational::facade::durability::RecoveredRelationalSemanticResult {
        self.native.effect().recovered_semantic_result()
    }
}
