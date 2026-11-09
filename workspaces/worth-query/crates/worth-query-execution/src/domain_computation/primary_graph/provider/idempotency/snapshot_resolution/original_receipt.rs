//! Original owner meaning has no live session, result, or delivery authority.
mod causality;
pub use causality::WorthQueryRecoveredAftermathCausality;
mod outbox;
use crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationCommitOutcomeIdentity;
pub use outbox::WorthQueryRecoveredDispatchOutbox;
use std::sync::Arc;
use worth_foundational::facade::{
    DurableQueryOutputSettlement, HomeIncarnation, RecordedCompositePublicationAttemptIdentity,
    RecordedProductBranchReference,
};
use worth_relational::facade::{
    branch::{RecordedRelationalComponentReference, RelationalBranchBasisDescriptor},
    durability::RecoveredRelationalSemanticResult,
    history::RelationalCommitReceipt,
    publication::PatchStreamPosition,
    transactions::RecordRef,
};

/// A read of the original committed occurrence, rebuilt from its actual owner
/// artifacts. Cloning retains funded immutable meaning, never live owner custody.
/// Live provider sessions, snapshot IDs, timings and delivery witnesses have
/// no representation here. Memory observations carry no Home publication fields.
///
/// ```
/// use worth_query_execution::facade::primary_graph::WorthQueryRecoveredApplicationCommitReceipt;
/// fn original_fields(value: &WorthQueryRecoveredApplicationCommitReceipt) {
///     let _ = (value.semantic_result().semantic(), value.old_product_head(), value.new_product_head(), value.recorded_intent(), value.exact_output_settlement());
/// }
/// ```
///
/// ```compile_fail,E0451
/// use worth_query_execution::facade::primary_graph::WorthQueryRecoveredApplicationCommitReceipt;
/// fn cannot_mint_from_an_index() -> WorthQueryRecoveredApplicationCommitReceipt {
///     WorthQueryRecoveredApplicationCommitReceipt { original: panic!() }
/// }
/// ```
///
/// ```compile_fail,E0599
/// use worth_query_execution::facade::primary_graph::{WorthQueryApplicationCommitReceipt, WorthQueryRecoveredApplicationCommitReceipt};
/// fn process_evidence_cannot_mint(value: WorthQueryApplicationCommitReceipt) -> WorthQueryRecoveredApplicationCommitReceipt {
///     WorthQueryRecoveredApplicationCommitReceipt::from_live(value)
/// }
/// ```
///
/// ```compile_fail,E0599
/// use worth_query_execution::facade::primary_graph::WorthQueryRecoveredApplicationCommitReceipt;
/// fn recovered_is_not_performed(value: &mut WorthQueryRecoveredApplicationCommitReceipt) {
///     value.take_performed_relational_product_change();
/// }
/// ```
#[derive(Clone, Debug)]
pub struct WorthQueryRecoveredApplicationCommitReceipt {
    original: Arc<RecordedReceipt>,
}

mod sealing;
use sealing::PublicationOrigin;
pub(super) use sealing::{MemoryOriginalReceiptInputs, OriginalReceiptInputs, OriginalReceiptRows};

#[derive(Debug)]
struct RecordedReceipt {
    binding: RecordedOriginalBinding,
    semantic: RecoveredRelationalSemanticResult,
    before_component: RecordedRelationalComponentReference,
    after_component: RecordedRelationalComponentReference,
    old_head: RecordedProductBranchReference,
    new_head: RecordedProductBranchReference,
    attempt: RecordedCompositePublicationAttemptIdentity,
    origin: PublicationOrigin,
    native_position: PatchStreamPosition,
    native_digest: [u8; 32],
    outbox: Option<WorthQueryRecoveredDispatchOutbox>,
    causality: Option<WorthQueryRecoveredAftermathCausality>,
    _funding: Option<worth_execution::ExecutionMemoryReservation>,
}

#[derive(Debug, Eq, PartialEq)]
pub(in crate::domain_computation::primary_graph::provider) struct RecordedOriginalBinding {
    pub(in crate::domain_computation::primary_graph::provider) key: [u8; 32],
    pub(in crate::domain_computation::primary_graph::provider) intent: String,
    pub(in crate::domain_computation::primary_graph::provider) affinity: String,
    pub(in crate::domain_computation::primary_graph::provider) outcome:
        WorthQueryApplicationCommitOutcomeIdentity,
    pub(in crate::domain_computation::primary_graph::provider) emitted: usize,
    pub(in crate::domain_computation::primary_graph::provider) settlement:
        DurableQueryOutputSettlement,
}

impl WorthQueryRecoveredApplicationCommitReceipt {
    pub(super) const fn retained_backing_bytes() -> usize {
        std::mem::size_of::<RecordedReceipt>() + 2 * std::mem::size_of::<usize>()
    }
    pub fn commit_reference(&self) -> &RelationalCommitReceipt {
        self.original.semantic.semantic().receipt()
    }
    pub fn native_journal_position(&self) -> PatchStreamPosition {
        self.original.native_position
    }
    pub fn native_artifact_digest(&self) -> &[u8; 32] {
        &self.original.native_digest
    }
    pub fn semantic_result(&self) -> &RecoveredRelationalSemanticResult {
        &self.original.semantic
    }
    pub fn changed_record_count(&self) -> usize {
        self.original.semantic.semantic().changed_records().len()
    }
    pub fn outcome_identity(&self) -> WorthQueryApplicationCommitOutcomeIdentity {
        self.original.binding.outcome
    }
    pub fn emitted_effect_count(&self) -> usize {
        self.original.binding.emitted
    }
    pub fn idempotency_key_identity(&self) -> &[u8; 32] {
        &self.original.binding.key
    }
    pub fn recorded_intent(&self) -> &str {
        &self.original.binding.intent
    }
    pub fn recorded_affinity(&self) -> &str {
        &self.original.binding.affinity
    }
    pub fn exact_output_settlement(&self) -> DurableQueryOutputSettlement {
        self.original.binding.settlement
    }
    pub fn basis_descriptor(&self) -> &RelationalBranchBasisDescriptor {
        self.original.after_component.descriptor()
    }
    pub fn recorded_component(&self) -> &RecordedRelationalComponentReference {
        &self.original.after_component
    }
    pub fn previous_recorded_component(&self) -> &RecordedRelationalComponentReference {
        &self.original.before_component
    }
    pub fn old_product_head(&self) -> &RecordedProductBranchReference {
        &self.original.old_head
    }
    pub fn new_product_head(&self) -> &RecordedProductBranchReference {
        &self.original.new_head
    }
    pub fn attempt_identity(&self) -> RecordedCompositePublicationAttemptIdentity {
        self.original.attempt
    }
    pub fn world_journal_position(&self) -> Option<u64> {
        match &self.original.origin {
            PublicationOrigin::Home { position, .. } => Some(*position),
            PublicationOrigin::MemoryOnly => None,
        }
    }
    pub fn world_artifact_digest(&self) -> Option<&[u8; 32]> {
        match &self.original.origin {
            PublicationOrigin::Home { digest, .. } => Some(digest),
            PublicationOrigin::MemoryOnly => None,
        }
    }
    pub fn publishing_incarnation(&self) -> Option<HomeIncarnation> {
        match &self.original.origin {
            PublicationOrigin::Home { publisher, .. } => Some(*publisher),
            PublicationOrigin::MemoryOnly => None,
        }
    }
    pub fn dispatch_outbox(&self) -> Option<&WorthQueryRecoveredDispatchOutbox> {
        self.original.outbox.as_ref()
    }
    pub fn dispatch_outbox_record(&self) -> Option<&RecordRef> {
        self.original
            .outbox
            .as_ref()
            .map(|record| record.record_ref())
    }
    pub fn aftermath_causality(&self) -> Option<&WorthQueryRecoveredAftermathCausality> {
        self.original.causality.as_ref()
    }
}

impl PartialEq for WorthQueryRecoveredApplicationCommitReceipt {
    fn eq(&self, other: &Self) -> bool {
        let (left, right) = (&self.original, &other.original);
        left.binding == right.binding
            && left.semantic.semantic() == right.semantic.semantic()
            && left.before_component == right.before_component
            && left.after_component == right.after_component
            && left.old_head == right.old_head
            && left.new_head == right.new_head
            && left.attempt == right.attempt
            && left.origin == right.origin
            && left.native_position == right.native_position
            && left.native_digest == right.native_digest
            && left.outbox == right.outbox
            && left.causality == right.causality
    }
}
impl Eq for WorthQueryRecoveredApplicationCommitReceipt {}
