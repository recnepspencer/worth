//! Read-only observation of a performed invalidation, without its authority.
use std::sync::Arc;

use crate::data::handle::NodeId;
use crate::data::retained_storage::SignalConditionalRetentionReservation;
use crate::data::telemetry::SignalInvalidationRealizedCounters;
use crate::logic::transaction::SignalObservationRequest;

#[derive(Clone)]
pub struct SignalInvalidationCausalObservation {
    backing: Arc<InvalidationObservationBacking>,
}

pub(super) struct InvalidationObservationBacking {
    graph_instance: u64,
    counters: SignalInvalidationRealizedCounters,
    executed_targets: Vec<NodeId>,
    request: SignalObservationRequest,
    _storage_custody: Option<SignalConditionalRetentionReservation>,
}

impl SignalInvalidationCausalObservation {
    pub(super) fn after_execution(
        graph_instance: u64,
        counters: SignalInvalidationRealizedCounters,
        executed_targets: Vec<NodeId>,
        request: SignalObservationRequest,
        storage_custody: Option<SignalConditionalRetentionReservation>,
    ) -> Self {
        Self {
            backing: Arc::new(InvalidationObservationBacking {
                graph_instance,
                counters,
                executed_targets,
                request,
                _storage_custody: storage_custody,
            }),
        }
    }

    pub fn same_observation(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.backing, &other.backing)
    }

    pub fn realized_counters(&self) -> &SignalInvalidationRealizedCounters {
        &self.backing.counters
    }

    pub fn retains_executed_target(&self, graph_instance: u64, target: NodeId) -> bool {
        self.backing.graph_instance == graph_instance
            && self.backing.executed_targets.binary_search(&target).is_ok()
    }

    pub fn request(&self) -> SignalObservationRequest {
        self.backing.request
    }
}

impl std::fmt::Debug for SignalInvalidationCausalObservation {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("SignalInvalidationCausalObservation")
            .finish_non_exhaustive()
    }
}
