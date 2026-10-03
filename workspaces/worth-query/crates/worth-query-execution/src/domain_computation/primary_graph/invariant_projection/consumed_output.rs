use std::mem::{align_of, size_of};
use std::sync::{Arc, OnceLock};

use worth_relational::facade::runtime::PositionedRelationalSnapshot;

use crate::domain_computation::execution_runtime::source_invalidation::RetainedInvalidationCapacity;
use crate::domain_computation::primary_graph::{
    application_attempt::WorthQueryApplicationObservedFact,
    output_lineage::{
        invalidation::{FullVerificationReason, RetainedConsumedOutputCapacity},
        RecordedSettlementIdentity, SealedNativeOutputWitness,
    },
};

#[cfg(feature = "certification-invalidation-equivalence")]
mod equivalence;
mod pending_dependency;
#[cfg(test)]
mod test_support;
mod verification;
pub(in crate::domain_computation::primary_graph) use pending_dependency::SelectedPendingConsumedOutput;
#[cfg(feature = "certification-invalidation-equivalence")]
pub(in crate::domain_computation::primary_graph) use verification::EvidenceView;
pub(in crate::domain_computation::primary_graph) use verification::{
    ConsumedOutputVerification, ConsumedOutputVerificationStop,
};

/// One output actually selected by a projected read. The lineage owner minted
/// the identity; Arc only shares retained evidence and is not an identity.
/// This runtime-only record is never serialized through a TypeId-bearing key.
#[derive(Clone, Debug)]
pub(in crate::domain_computation::primary_graph) struct ConsumedOutputEvidence {
    identity: Arc<RecordedSettlementIdentity>,
    source_facts: Arc<[WorthQueryApplicationObservedFact]>,
    upstream: Arc<[ConsumedOutputEvidence]>,
    verification_requirement: Option<FullVerificationReason>,
    native_output_witness: Option<Arc<OnceLock<SealedNativeOutputWitness>>>,
    selected_native_root: Arc<PositionedRelationalSnapshot>,
    _capacity: RetainedConsumedOutputCapacity,
    backing_capacity: Option<Arc<RetainedInvalidationCapacity>>,
}

impl PartialEq for ConsumedOutputEvidence {
    fn eq(&self, other: &Self) -> bool {
        self.identity == other.identity
            && self.source_facts == other.source_facts
            && self.upstream == other.upstream
            && self.verification_requirement == other.verification_requirement
            && match (&self.native_output_witness, &other.native_output_witness) {
                (Some(left), Some(right)) => Arc::ptr_eq(left, right),
                (None, None) => true,
                _ => false,
            }
            && self.selected_native_root == other.selected_native_root
    }
}

impl Eq for ConsumedOutputEvidence {}

impl ConsumedOutputEvidence {
    pub(super) fn new(
        identity: Arc<RecordedSettlementIdentity>,
        source_facts: Arc<[WorthQueryApplicationObservedFact]>,
        upstream: Arc<[ConsumedOutputEvidence]>,
        verification_requirement: Option<FullVerificationReason>,
        native_output_witness: Arc<OnceLock<SealedNativeOutputWitness>>,
        selected_native_root: Arc<PositionedRelationalSnapshot>,
        capacity: RetainedConsumedOutputCapacity,
    ) -> Self {
        Self {
            identity,
            source_facts,
            upstream,
            verification_requirement,
            native_output_witness: Some(native_output_witness),
            selected_native_root,
            _capacity: capacity,
            backing_capacity: None,
        }
    }

    pub(super) const fn metadata_bytes() -> u64 {
        (size_of::<Self>()
            + size_of::<RecordedSettlementIdentity>()
            + size_of::<usize>() * 2
            + align_of::<RecordedSettlementIdentity>() * 2) as u64
    }

    pub(in crate::domain_computation::primary_graph) fn attach_backing_capacity(
        &mut self,
        capacity: Arc<RetainedInvalidationCapacity>,
    ) {
        self.backing_capacity = Some(capacity);
    }

    pub(in crate::domain_computation::primary_graph) fn identity(
        &self,
    ) -> &Arc<RecordedSettlementIdentity> {
        &self.identity
    }
}
