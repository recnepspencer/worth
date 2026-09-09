use worth_query_installation::facade::ApplicationSchema;

use super::WorthQueryConditionalRuntimeInstallationDenial;
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorthQueryConditionalRuntimeInspection {
    installed_binding_count: usize,
    managed_clock_count: usize,
    retained_wake_count: usize,
    reconstructed_intent_count: usize,
    provider_count: usize,
    lease_count: usize,
    retained_attempt_count: usize,
    retained_direct_delivery_count: usize,
    scheduler_task_count: usize,
    scheduler_queue_count: usize,
    signal_graph_count: usize,
    installation_canonical_work: worth_query_installation::facade::WorthQueryCanonicalWorkEvidence,
}

impl WorthQueryConditionalRuntimeInspection {
    pub(in crate::domain_computation::primary_graph) const fn from_live_resources(
        installed_binding_count: usize,
        managed_clock_count: usize,
        retained_wake_count: usize,
        reconstructed_intent_count: usize,
        provider_count: usize,
        lease_count: usize,
        retained_attempt_count: usize,
        retained_direct_delivery_count: usize,
        signal_graph_count: usize,
        installation_canonical_work: worth_query_installation::facade::WorthQueryCanonicalWorkEvidence,
    ) -> Self {
        Self {
            installed_binding_count,
            managed_clock_count,
            retained_wake_count,
            reconstructed_intent_count,
            provider_count,
            lease_count,
            retained_attempt_count,
            retained_direct_delivery_count,
            scheduler_task_count: 0,
            scheduler_queue_count: 0,
            signal_graph_count,
            installation_canonical_work,
        }
    }

    pub const fn installed_binding_count(self) -> usize {
        self.installed_binding_count
    }

    pub const fn managed_clock_count(self) -> usize {
        self.managed_clock_count
    }

    pub const fn retained_wake_count(self) -> usize {
        self.retained_wake_count
    }

    pub const fn reconstructed_intent_count(self) -> usize {
        self.reconstructed_intent_count
    }

    pub const fn provider_count(self) -> usize {
        self.provider_count
    }

    pub const fn lease_count(self) -> usize {
        self.lease_count
    }

    pub const fn retained_attempt_count(self) -> usize {
        self.retained_attempt_count
    }

    pub const fn retained_direct_delivery_count(self) -> usize {
        self.retained_direct_delivery_count
    }

    pub const fn scheduler_task_count(self) -> usize {
        self.scheduler_task_count
    }

    pub const fn scheduler_queue_count(self) -> usize {
        self.scheduler_queue_count
    }

    pub const fn signal_graph_count(self) -> usize {
        self.signal_graph_count
    }

    pub const fn installation_canonical_work(
        self,
    ) -> worth_query_installation::facade::WorthQueryCanonicalWorkEvidence {
        self.installation_canonical_work
    }

    pub const fn is_empty(self) -> bool {
        self.installed_binding_count == 0
            && self.managed_clock_count == 0
            && self.retained_wake_count == 0
            && self.reconstructed_intent_count == 0
            && self.provider_count == 0
            && self.lease_count == 0
            && self.retained_attempt_count == 0
            && self.retained_direct_delivery_count == 0
            && self.scheduler_task_count == 0
            && self.scheduler_queue_count == 0
            && self.signal_graph_count == 0
    }
}

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: ApplicationSchema,
{
    pub fn inspect_conditional_runtime(&self) -> WorthQueryConditionalRuntimeInspection {
        let registry = self
            .conditional_operations
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .snapshot();
        let retained = registry.retained_resource_counts();
        let bridge = self.bridge.conditional().conditional_lifecycle_probe();
        WorthQueryConditionalRuntimeInspection::from_live_resources(
            registry.len(),
            bridge.live_managed_clock_count(),
            retained.wakes,
            retained.intents,
            bridge.live_provider_count(),
            registry.len(),
            retained.attempts,
            retained.direct_deliveries,
            bridge.live_signal_graph_count(),
            registry.installation_canonical_work(),
        )
    }

    pub fn conditional_runtime_lifecycle_probe(
        &self,
    ) -> super::WorthQueryConditionalRuntimeLifecycleProbe {
        let bridge = self.bridge.conditional().conditional_lifecycle_probe();
        let registry = self
            .conditional_operations
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .snapshot();
        registry.lifecycle_probe(bridge)
    }

    pub fn close_conditional_runtime(
        &mut self,
    ) -> Result<
        WorthQueryConditionalRuntimeInspection,
        WorthQueryConditionalRuntimeInstallationDenial,
    > {
        let before = self.inspect_conditional_runtime();
        self.release_conditional_runtime_resources();
        Ok(before)
    }
}
