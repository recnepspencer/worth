/// Query-owned evidence that a fresh output patch reached its declared
/// readiness conditional and produced a Signal successor.
#[derive(Clone)]
pub struct WorthQueryOutputReadinessDeliveryEvidence {
    // `from_execution` is reached only after the installed producer boundary
    // returned the receipt whose readiness is evaluated.
    producer_contacts: usize,
    // A Bridge delivery contact exists only when that receipt carried a fresh
    // performed change into readiness evaluation.
    delivery_contacts: usize,
    conditional_successor: bool,
    truth_targets_admitted: usize,
    signal_seeds_emitted: usize,
    slots_touched: usize,
    signal_decision: crate::domain_computation::primary_graph::WorthQueryConditionalSignalDecision,
    semantic_observation_reads: usize,
}

impl WorthQueryOutputReadinessDeliveryEvidence {
    pub(in crate::domain_computation::primary_graph) const fn without_execution() -> Self {
        Self {
            producer_contacts: 0,
            delivery_contacts: 0,
            conditional_successor: false,
            truth_targets_admitted: 0,
            signal_seeds_emitted: 0,
            slots_touched: 0,
            signal_decision: crate::domain_computation::primary_graph::WorthQueryConditionalSignalDecision::DependencyUnchanged,
            semantic_observation_reads: 0,
        }
    }

    #[cfg(test)]
    pub(in crate::domain_computation::primary_graph) const fn for_test() -> Self {
        Self {
            producer_contacts: 1,
            delivery_contacts: 0,
            conditional_successor: false,
            truth_targets_admitted: 0,
            signal_seeds_emitted: 0,
            slots_touched: 0,
            signal_decision: crate::domain_computation::primary_graph::WorthQueryConditionalSignalDecision::DependencyUnchanged,
            semantic_observation_reads: 1,
        }
    }

    pub(in crate::domain_computation::primary_graph) fn from_execution(
        delivery: Option<&worth_runtime_bridge::facade::BridgeCorrespondenceDeliveryReceipt>,
        execution: &worth_runtime_bridge::facade::BridgeConditionalDecisionEvidence,
    ) -> Self {
        Self {
            producer_contacts: 1,
            delivery_contacts: usize::from(delivery.is_some()),
            conditional_successor: delivery.is_some_and(|receipt| receipt.has_conditional_successor()),
            truth_targets_admitted: delivery.map_or(0, |receipt| receipt.truth_targets_admitted()),
            signal_seeds_emitted: delivery.map_or(0, |receipt| receipt.signal_seeds_emitted()),
            slots_touched: delivery.map_or(0, |receipt| receipt.slots_touched()),
            signal_decision: crate::domain_computation::primary_graph::conditional_operation::classify_bridge_signal(execution),
            semantic_observation_reads: execution.semantic_observation_reads(),
        }
    }

    pub const fn producer_contact_count(&self) -> usize {
        self.producer_contacts
    }

    pub const fn delivery_contact_count(&self) -> usize {
        self.delivery_contacts
    }

    pub const fn has_conditional_successor(&self) -> bool {
        self.conditional_successor
    }

    pub const fn truth_targets_admitted(&self) -> usize {
        self.truth_targets_admitted
    }

    pub const fn signal_seeds_emitted(&self) -> usize {
        self.signal_seeds_emitted
    }

    pub const fn slots_touched(&self) -> usize {
        self.slots_touched
    }

    pub const fn signal_decision(
        &self,
    ) -> crate::domain_computation::primary_graph::WorthQueryConditionalSignalDecision {
        self.signal_decision
    }

    pub const fn semantic_observation_reads(&self) -> usize {
        self.semantic_observation_reads
    }
}
