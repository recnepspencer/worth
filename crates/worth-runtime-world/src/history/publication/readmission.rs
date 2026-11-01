//! Immutable recovered meaning owns the original Native admission exactly once.
use crate::{
    branch::ProductBranchReferenceSnapshot, identity::CompositePublicationAttemptIdentity,
};
use std::sync::Arc;
use worth_proof::RelationalPublicationReadmitted;
use worth_relational::facade::{
    durability::RelationalRecoveredPublicationCustody, transactions::RelationalCommitSemanticResult,
};

#[derive(Debug)]
pub(crate) struct ReadmittedPublicationRecord {
    pub(crate) before: ProductBranchReferenceSnapshot,
    pub(crate) after: ProductBranchReferenceSnapshot,
    pub(crate) attempt: CompositePublicationAttemptIdentity,
    pub(crate) location: crate::durability::AdmittedWorldOccurrenceLocation,
    pub(crate) incarnation: worth_foundational::facade::HomeIncarnation,
    native: RelationalRecoveredPublicationCustody,
}
impl ReadmittedPublicationRecord {
    pub(crate) fn new(
        before: ProductBranchReferenceSnapshot,
        after: ProductBranchReferenceSnapshot,
        attempt: CompositePublicationAttemptIdentity,
        location: crate::durability::AdmittedWorldOccurrenceLocation,
        incarnation: worth_foundational::facade::HomeIncarnation,
        native: RelationalPublicationReadmitted<RelationalRecoveredPublicationCustody>,
    ) -> Arc<Self> {
        Arc::new(Self {
            before,
            after,
            attempt,
            location,
            incarnation,
            native: native.consume(|native| native),
        })
    }
    pub(crate) fn semantic(&self) -> &RelationalCommitSemanticResult {
        self.native.effect().semantic_result()
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
    ForeignHome,
    OwnerUnavailable,
    History(crate::history::CompositeHistoryCatalogDenial),
    NotReadmitted,
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
        self.record.location.position()
    }
    pub fn artifact_digest(&self) -> &[u8; 32] {
        self.record.location.digest()
    }
    pub fn native_patch_position(
        &self,
    ) -> worth_relational::facade::publication::PatchStreamPosition {
        self.record.native.effect().subject().position()
    }
    pub fn native_artifact_digest(&self) -> &[u8; 32] {
        self.record.native.effect().subject().artifact_digest()
    }
    pub fn admitted_incarnation(&self) -> worth_foundational::facade::HomeIncarnation {
        self.record.incarnation
    }
}
