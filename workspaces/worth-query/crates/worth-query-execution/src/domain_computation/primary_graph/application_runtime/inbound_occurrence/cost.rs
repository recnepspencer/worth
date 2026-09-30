//! Read-only, operation-scoped cost evidence for the signed inbound lane.

use std::sync::atomic::{AtomicU64, Ordering};

use super::{WorthQueryInboundVerifierHandle, WorthQueryPrimaryGraphApplicationRuntime};

#[derive(Default)]
pub(in crate::domain_computation::primary_graph) struct WorthQueryInboundCostLedger {
    pub(in crate::domain_computation::primary_graph) verifier_input_bytes: AtomicU64,
    pub(in crate::domain_computation::primary_graph) custody_key_probes: AtomicU64,
    pub(in crate::domain_computation::primary_graph) terminal_key_probes: AtomicU64,
    pub(in crate::domain_computation::primary_graph) outbox_key_probes: AtomicU64,
    pub(in crate::domain_computation::primary_graph) selected_outbox_records: AtomicU64,
    pub(in crate::domain_computation::primary_graph) completion_candidate_prepares: AtomicU64,
    pub(in crate::domain_computation::primary_graph) world_publication_attempts: AtomicU64,
    pub(in crate::domain_computation::primary_graph) world_performed_publications: AtomicU64,
}

/// Monotonic signed-inbound work and current owner custody for one installed operation.
/// Outstanding dispatches reflect the shared capacity pool for this operation's
/// installed inbound contract; equal contracts can share that pool.
/// Key probes count exact lookups, rather than implementation-specific tree comparisons.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct WorthQueryInboundCostObservation {
    verifier_input_bytes: u64,
    custody_key_probes: u64,
    terminal_key_probes: u64,
    outbox_key_probes: u64,
    selected_outbox_records: u64,
    completion_candidate_prepares: u64,
    world_publication_attempts: u64,
    world_performed_publications: u64,
    accepted_occurrences: u64,
    accepted_charged_bytes: u64,
    outstanding_dispatches: u64,
}

impl WorthQueryInboundCostObservation {
    pub const fn verifier_input_bytes(self) -> u64 {
        self.verifier_input_bytes
    }
    pub const fn custody_key_probes(self) -> u64 {
        self.custody_key_probes
    }
    pub const fn terminal_key_probes(self) -> u64 {
        self.terminal_key_probes
    }
    pub const fn outbox_key_probes(self) -> u64 {
        self.outbox_key_probes
    }
    pub const fn selected_outbox_records(self) -> u64 {
        self.selected_outbox_records
    }
    pub const fn completion_candidate_prepares(self) -> u64 {
        self.completion_candidate_prepares
    }
    pub const fn world_publication_attempts(self) -> u64 {
        self.world_publication_attempts
    }
    pub const fn world_performed_publications(self) -> u64 {
        self.world_performed_publications
    }
    pub const fn accepted_occurrences(self) -> u64 {
        self.accepted_occurrences
    }
    pub const fn accepted_charged_bytes(self) -> u64 {
        self.accepted_charged_bytes
    }
    /// Current dispatches charged to this installed inbound contract's capacity pool.
    pub const fn outstanding_dispatches(self) -> u64 {
        self.outstanding_dispatches
    }
}

impl<Schema: worth_query_installation::facade::ApplicationSchema>
    WorthQueryPrimaryGraphApplicationRuntime<Schema>
{
    /// The handle selects the installed operation; correlation bytes cannot select a cost scope.
    pub fn observe_inbound_cost(
        &self,
        handle: &WorthQueryInboundVerifierHandle,
    ) -> Option<WorthQueryInboundCostObservation> {
        let installed = self.installed_inbound_verifier(handle)?;
        let cost = &installed.cost;
        let (accepted_occurrences, accepted_charged_bytes) = self
            .inbound_custody
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .usage_for_operation(handle.operation());
        Some(WorthQueryInboundCostObservation {
            verifier_input_bytes: cost.verifier_input_bytes.load(Ordering::Relaxed),
            custody_key_probes: cost.custody_key_probes.load(Ordering::Relaxed),
            terminal_key_probes: cost.terminal_key_probes.load(Ordering::Relaxed),
            outbox_key_probes: cost.outbox_key_probes.load(Ordering::Relaxed),
            selected_outbox_records: cost.selected_outbox_records.load(Ordering::Relaxed),
            completion_candidate_prepares: cost
                .completion_candidate_prepares
                .load(Ordering::Relaxed),
            world_publication_attempts: cost.world_publication_attempts.load(Ordering::Relaxed),
            world_performed_publications: cost.world_performed_publications.load(Ordering::Relaxed),
            accepted_occurrences,
            accepted_charged_bytes,
            outstanding_dispatches: self
                .primary_provider
                .outstanding_dispatch_count_for_operation(&installed.contract),
        })
    }

    pub(in crate::domain_computation::primary_graph) fn inbound_cost_for_operation(
        &self,
        operation: &str,
    ) -> Option<std::sync::Arc<super::WorthQueryInstalledInboundVerifier>> {
        self.inbound_verifiers
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .get(operation)
            .cloned()
    }
}
