use worth_proof::{ActionMarker, Performed};

use crate::data::telemetry::SignalInvalidationRealizedCounters;
use crate::logic::transaction::{SignalObservationRequest, SignalObservationSession};

worth_proof::authority_marker!(InvalidationPerformedReceiptAuthority);

struct CompleteInvalidationExecutionObservation;
impl ActionMarker for CompleteInvalidationExecutionObservation {}

type PerformedInvalidationObservation = Performed<
    CompleteInvalidationExecutionObservation,
    InvalidationPerformedReceiptAuthority,
    SignalInvalidationRealizedCounters,
>;

/// Proof that Signal observed these counters after performed execution.
#[derive(Debug)]
pub struct SignalInvalidationExecutionReceipt {
    performed: PerformedInvalidationObservation,
    graph_instance: u64,
    executed_targets: Vec<crate::data::handle::NodeId>,
    request: SignalObservationRequest,
    _storage_custody: Option<crate::data::retained_storage::SignalConditionalRetentionReservation>,
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
                counters,
            ),
            graph_instance,
            executed_targets,
            request,
            _storage_custody: storage_custody,
        }
    }

    pub fn realized_counters(&self) -> &SignalInvalidationRealizedCounters {
        self.performed.outcome()
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
        self.graph_instance == graph_instance
            && self.executed_targets.binary_search(&target).is_ok()
    }

    pub const fn request(&self) -> SignalObservationRequest {
        self.request
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
