use std::sync::Weak;

use super::inspection::WorthQueryConditionalRuntimeInspection;

pub(in crate::domain_computation::primary_graph) struct WorthQueryConditionalOperationLiveness {
    pub(super) binding: Weak<()>,
    pub(super) lease: Weak<super::installation::ConditionalClockLease>,
    pub(super) wakes: Vec<Weak<()>>,
    pub(super) intents: Vec<Weak<()>>,
    pub(super) attempts: Vec<Weak<()>>,
}

pub struct WorthQueryConditionalRuntimeLifecycleProbe {
    bindings: Vec<Weak<()>>,
    leases: Vec<Weak<super::installation::ConditionalClockLease>>,
    wakes: Vec<Weak<()>>,
    intents: Vec<Weak<()>>,
    attempts: Vec<Weak<()>>,
    bridge: worth_runtime_bridge::facade::BridgeConditionalRuntimeLifecycleProbe,
}

impl WorthQueryConditionalRuntimeLifecycleProbe {
    pub(in crate::domain_computation::primary_graph) fn from_resources(
        operations: impl IntoIterator<Item = WorthQueryConditionalOperationLiveness>,
        bridge: worth_runtime_bridge::facade::BridgeConditionalRuntimeLifecycleProbe,
    ) -> Self {
        let mut probe = Self {
            bindings: Vec::new(),
            leases: Vec::new(),
            wakes: Vec::new(),
            intents: Vec::new(),
            attempts: Vec::new(),
            bridge,
        };
        for operation in operations {
            probe.bindings.push(operation.binding);
            probe.leases.push(operation.lease);
            probe.wakes.extend(operation.wakes);
            probe.intents.extend(operation.intents);
            probe.attempts.extend(operation.attempts);
        }
        probe
    }

    /// Reads the resources that remain live from the exact owners captured by
    /// this probe. No lifecycle hook writes or clears these observations.
    pub fn live_inventory(&self) -> WorthQueryConditionalRuntimeInspection {
        WorthQueryConditionalRuntimeInspection::from_live_resources(
            live_count(&self.bindings),
            self.bridge.live_managed_clock_count(),
            live_count(&self.wakes),
            live_count(&self.intents),
            self.bridge.live_provider_count(),
            live_count(&self.leases),
            live_count(&self.attempts),
            0,
            self.bridge.live_signal_graph_count(),
            worth_query_installation::facade::WorthQueryCanonicalWorkEvidence::zero(),
        )
    }
}

fn live_count<Resource>(resources: &[Weak<Resource>]) -> usize {
    resources
        .iter()
        .filter(|resource| resource.strong_count() != 0)
        .count()
}

pub(super) fn retained_resource_counts(
    wakes: &[super::signal_decision_reentry::WorthQueryRetainedConditionalWake],
    intents: usize,
) -> super::lifecycle::WorthQueryConditionalRetainedResourceCounts {
    super::lifecycle::WorthQueryConditionalRetainedResourceCounts {
        wakes: wakes.len(),
        intents,
        attempts: wakes
            .iter()
            .filter(|wake| wake.application_attempted)
            .count(),
        direct_deliveries: 0,
    }
}
