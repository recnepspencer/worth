use std::sync::{Arc, Weak};

use super::{BridgeInstalledConditionalLowering, BridgeOwnedSignalRuntime};

/// Weak observation of the concrete Bridge and Signal resources owned by one
/// conditional runtime at the instant the probe is acquired.
pub struct BridgeConditionalRuntimeLifecycleProbe {
    signal_graph: Option<worth_signal::facade::SignalGraphLifecycleProbe>,
    conditional_port: Option<super::signal_port::BridgeConditionalSignalPort>,
    providers: Vec<Weak<BridgeInstalledConditionalLowering>>,
    managed_clocks: Vec<Weak<()>>,
    retention: Weak<super::retention::BridgeRetentionLedger>,
}

impl BridgeConditionalRuntimeLifecycleProbe {
    pub fn live_signal_graph_count(&self) -> usize {
        self.signal_graph
            .as_ref()
            .is_some_and(worth_signal::facade::SignalGraphLifecycleProbe::is_live)
            .into()
    }

    pub fn live_provider_count(&self) -> usize {
        self.providers
            .iter()
            .filter(|provider| provider.strong_count() != 0)
            .count()
    }

    pub fn live_managed_clock_count(&self) -> usize {
        self.managed_clocks
            .iter()
            .filter(|clock| clock.strong_count() != 0)
            .count()
    }

    pub fn signal_conditional_retention(
        &self,
    ) -> Option<worth_signal::facade::branch::SignalConditionalRetentionObservation> {
        self.conditional_port.as_ref()?.retention_observation().ok()
    }

    pub fn bridge_conditional_retention(
        &self,
    ) -> Option<super::retention::BridgeConditionalRetentionObservation> {
        self.retention.upgrade().map(|ledger| ledger.observation())
    }
}

impl BridgeOwnedSignalRuntime {
    pub fn conditional_lifecycle_probe(&self) -> BridgeConditionalRuntimeLifecycleProbe {
        let providers = self
            .conditional_lowerings
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .values()
            .map(Arc::downgrade)
            .collect();
        let clocks: Vec<_> = self
            .managed_clock_lanes
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .values()
            .cloned()
            .collect();
        let managed_clocks = clocks
            .iter()
            .map(|cell| {
                let lane = cell
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                Arc::downgrade(&lane.lifecycle_token)
            })
            .collect();
        BridgeConditionalRuntimeLifecycleProbe {
            signal_graph: Some(self.signal_graph_lifecycle_probe.clone()),
            conditional_port: self.signal_services.as_ref().map(
                super::service_binding::BridgeSignalServiceBinding::conditional_extension_port,
            ),
            providers,
            managed_clocks,
            retention: Arc::downgrade(&self.retention),
        }
    }

    /// Releases conditional resources while Runtime World retains the sealed owner.
    pub fn close_conditional_resources(&mut self) {
        self.retention.close();
        let mut lowerings = self
            .conditional_lowerings
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let clocks = std::mem::take(
            self.managed_clock_lanes
                .get_mut()
                .unwrap_or_else(std::sync::PoisonError::into_inner),
        );
        for lowering in lowerings.values() {
            lowering.lease.revoke_liveness();
        }
        lowerings.replace_installed(Default::default());
        for clock in clocks.values() {
            clock
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .revoke_liveness();
        }
        *self
            .owned_conditional_targets
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = Default::default();
        drop(clocks);
        drop(lowerings);
    }
}
