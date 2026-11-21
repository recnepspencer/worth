/// Execution-owned observation of lower-runtime granular delivery.
///
/// These rows describe transport performed by the primary runtime. They do
/// not predict or authorize Query maintenance.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct WorthQueryGranularInvalidationObservation {
    direct_truth_delivery_count: usize,
    signal_performed_delivery_count: usize,
    bridge: WorthQueryBridgeGranularDeliveryCounters,
    signal: worth_signal::facade::adapters::SignalInvalidationRealizedCounters,
}

impl WorthQueryGranularInvalidationObservation {
    pub(crate) fn from_deliveries(
        deliveries: &[worth_runtime_bridge::facade::BridgeGranularInvalidationDelivery],
    ) -> Self {
        let mut bridge = WorthQueryBridgeGranularDeliveryCounters::default();
        let mut signal_values = [0_u64; 24];
        let mut signal_performed_delivery_count = 0;
        for delivery in deliveries {
            bridge.add(delivery.correspondence_receipt().counters());
            let Some(performed) = delivery.performed_signal() else {
                continue;
            };
            signal_performed_delivery_count += 1;
            for (total, value) in signal_values
                .iter_mut()
                .zip(performed.performed().realized_counters().values())
            {
                *total = total.saturating_add(value);
            }
        }
        Self {
            direct_truth_delivery_count: deliveries.len(),
            signal_performed_delivery_count,
            bridge,
            signal: worth_signal::facade::adapters::SignalInvalidationRealizedCounters::from_values(
                signal_values,
            ),
        }
    }

    pub const fn direct_truth_delivery_count(self) -> usize {
        self.direct_truth_delivery_count
    }

    pub const fn signal_performed_delivery_count(self) -> usize {
        self.signal_performed_delivery_count
    }

    pub const fn bridge_performed(self) -> WorthQueryBridgeGranularDeliveryCounters {
        self.bridge
    }

    pub const fn signal_performed(
        self,
    ) -> worth_signal::facade::adapters::SignalInvalidationRealizedCounters {
        self.signal
    }
}

/// Work performed by Runtime Bridge while producing one granular batch.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct WorthQueryBridgeGranularDeliveryCounters {
    pub source_load_attempts: usize,
    pub source_envelopes_loaded: usize,
    pub allocation_registry_lock_attempts: usize,
    pub allocation_source_set_checks: usize,
    pub signal_basis_target_checks: usize,
    pub signal_capability_admissions: usize,
    pub failed_deliveries: usize,
    pub truth_targets_admitted: usize,
    pub correspondence_lookups: usize,
    pub semantic_match_checks: usize,
    pub aspect_rejections: usize,
    pub binding_rejections: usize,
    pub change_kind_rejections: usize,
    pub locality_rejections: usize,
    pub projection_rejections: usize,
    pub relevant_change_checks: usize,
    pub projection_paths_inspected: usize,
    pub source_widening_target_checks: usize,
    pub signal_seeds_emitted: usize,
    pub node_fan_out: usize,
    pub slots_touched: usize,
}

impl WorthQueryBridgeGranularDeliveryCounters {
    fn add(&mut self, counters: worth_runtime_bridge::facade::CorrespondenceDeliveryCounters) {
        self.source_load_attempts += counters.source_load_attempts();
        self.source_envelopes_loaded += counters.source_envelopes_loaded();
        self.allocation_registry_lock_attempts += counters.allocation_registry_lock_attempts();
        self.allocation_source_set_checks += counters.allocation_source_set_checks();
        self.signal_basis_target_checks += counters.signal_basis_target_checks();
        self.signal_capability_admissions += counters.signal_capability_admissions();
        self.failed_deliveries += counters.failed_deliveries();
        self.truth_targets_admitted += counters.truth_targets_admitted();
        self.correspondence_lookups += counters.correspondence_lookups();
        self.semantic_match_checks += counters.semantic_match_checks();
        self.aspect_rejections += counters.aspect_rejections();
        self.binding_rejections += counters.binding_rejections();
        self.change_kind_rejections += counters.change_kind_rejections();
        self.locality_rejections += counters.locality_rejections();
        self.projection_rejections += counters.projection_rejections();
        self.relevant_change_checks += counters.relevant_change_checks();
        self.projection_paths_inspected += counters.projection_paths_inspected();
        self.source_widening_target_checks += counters.source_widening_target_checks();
        self.signal_seeds_emitted += counters.signal_seeds_emitted();
        self.node_fan_out += counters.node_fan_out();
        self.slots_touched += counters.slots_touched();
    }
}
