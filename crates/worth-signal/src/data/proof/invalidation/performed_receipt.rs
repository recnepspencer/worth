use worth_proof::{ActionMarker, Performed};

use crate::data::telemetry::SignalInvalidationRealizedCounters;
use crate::logic::transaction::{SignalObservationRequest, SignalObservationSession};

mod observation;
pub use observation::SignalInvalidationCausalObservation;

worth_proof::authority_marker!(InvalidationPerformedReceiptAuthority);

struct CompleteInvalidationExecutionObservation;
impl ActionMarker for CompleteInvalidationExecutionObservation {}

type PerformedInvalidationObservation = Performed<
    CompleteInvalidationExecutionObservation,
    InvalidationPerformedReceiptAuthority,
    SignalInvalidationCausalObservation,
>;

/// Proof that Signal observed these counters after performed execution.
#[derive(Debug)]
pub struct SignalInvalidationExecutionReceipt {
    performed: PerformedInvalidationObservation,
}

/// Backwards-compatible name for the managed Signal observation session.
pub type SignalInvalidationExecutionObservation = SignalObservationSession;

impl SignalInvalidationExecutionReceipt {
    pub(crate) fn after_execution(
        graph_instance: u64,
        counters: SignalInvalidationRealizedCounters,
        executed_targets: Vec<crate::data::handle::NodeId>,
        request: SignalObservationRequest,
        storage_custody: Option<
            crate::data::retained_storage::SignalConditionalRetentionReservation,
        >,
    ) -> Self {
        Self {
            performed: Performed::record(
                &InvalidationPerformedReceiptAuthority::witness(),
                SignalInvalidationCausalObservation::after_execution(
                    graph_instance,
                    counters,
                    executed_targets,
                    request,
                    storage_custody,
                ),
            ),
        }
    }

    pub(crate) fn retained_header_charge() -> Result<
        crate::data::retained_storage::RetainedStorageCharge,
        crate::data::retained_storage::RetainedStoragePreparationDenial,
    > {
        use crate::data::retained_storage::{arc_allocation_charge, RetainedStorageCharge};
        RetainedStorageCharge::capacity::<Self>(1)?.checked_add(arc_allocation_charge::<
            observation::InvalidationObservationBacking,
        >()?)
    }

    /// Share the actual immutable observation, without cloning Performed authority.
    pub fn retain_causal_observation(&self) -> SignalInvalidationCausalObservation {
        self.performed.outcome().clone()
    }

    pub fn realized_counters(&self) -> &SignalInvalidationRealizedCounters {
        self.performed.outcome().realized_counters()
    }

    /// Produce a descriptive summary from this performed observation.
    pub fn summary(&self) -> InvalidationExecutionSummary {
        InvalidationExecutionSummary {
            realized_counters: *self.realized_counters(),
        }
    }

    pub fn retains_executed_target(
        &self,
        graph_instance: u64,
        target: crate::data::handle::NodeId,
    ) -> bool {
        self.performed
            .outcome()
            .retains_executed_target(graph_instance, target)
    }

    pub fn request(&self) -> SignalObservationRequest {
        self.performed.outcome().request()
    }

    pub const fn completion(&self) -> crate::logic::transaction::SignalObservationCompletion {
        crate::logic::transaction::SignalObservationCompletion::Completed
    }
}

/// Read-only summary derived from performed invalidation evidence.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InvalidationExecutionSummary {
    realized_counters: SignalInvalidationRealizedCounters,
}

impl InvalidationExecutionSummary {
    pub const fn realized_counters(&self) -> &SignalInvalidationRealizedCounters {
        &self.realized_counters
    }
}

impl crate::data::proof::SummaryForm for InvalidationExecutionSummary {}
