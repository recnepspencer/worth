use worth_query_declaration::facade::application_program::ApplicationWorkflowComponentLimits;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorthQueryApplicationWorkflowResourceCeiling {
    maximum_definition_nodes: u16,
    maximum_definition_connections: u16,
    maximum_definition_effects: u16,
    component_limits: ApplicationWorkflowComponentLimits,
    maximum_canonical_bytes: u32,
    maximum_live_instances: u32,
    maximum_retained_transitions_per_instance: u32,
    maximum_evidence_bytes: u64,
    history_reconstruction: WorthQueryWorkflowHistoryReconstructionBudget,
}

impl WorthQueryApplicationWorkflowResourceCeiling {
    pub const fn new(
        maximum_definition_nodes: u16,
        maximum_definition_connections: u16,
        maximum_definition_effects: u16,
        component_limits: ApplicationWorkflowComponentLimits,
        maximum_canonical_bytes: u32,
        maximum_live_instances: u32,
        maximum_retained_transitions_per_instance: u32,
        maximum_evidence_bytes: u64,
    ) -> Option<Self> {
        if maximum_definition_nodes == 0
            || maximum_definition_connections == 0
            || maximum_definition_effects == 0
            || maximum_canonical_bytes == 0
            || maximum_live_instances == 0
            || maximum_retained_transitions_per_instance == 0
            || maximum_evidence_bytes == 0
        {
            None
        } else {
            Some(Self {
                maximum_definition_nodes,
                maximum_definition_connections,
                maximum_definition_effects,
                component_limits,
                maximum_canonical_bytes,
                maximum_live_instances,
                maximum_retained_transitions_per_instance,
                maximum_evidence_bytes,
                history_reconstruction: WorthQueryWorkflowHistoryReconstructionBudget::standard(),
            })
        }
    }

    pub const fn maximum_definition_nodes(self) -> u16 {
        self.maximum_definition_nodes
    }

    pub const fn maximum_definition_connections(self) -> u16 {
        self.maximum_definition_connections
    }

    pub const fn maximum_definition_effects(self) -> u16 {
        self.maximum_definition_effects
    }

    pub const fn maximum_component_depth(self) -> u8 {
        self.component_limits.maximum_depth()
    }

    pub const fn component_limits(self) -> ApplicationWorkflowComponentLimits {
        self.component_limits
    }

    pub const fn maximum_canonical_bytes(self) -> u32 {
        self.maximum_canonical_bytes
    }

    pub const fn maximum_live_instances(self) -> u32 {
        self.maximum_live_instances
    }

    pub const fn maximum_retained_transitions_per_instance(self) -> u32 {
        self.maximum_retained_transitions_per_instance
    }

    pub const fn maximum_evidence_bytes(self) -> u64 {
        self.maximum_evidence_bytes
    }
    /// Cold and replay-history reads have their own ceilings, independent of
    /// ordinary decision facts and authoritative history retention.
    pub const fn with_history_reconstruction_budget(
        mut self,
        budget: WorthQueryWorkflowHistoryReconstructionBudget,
    ) -> Self {
        self.history_reconstruction = budget;
        self
    }

    pub const fn history_reconstruction_budget(
        self,
    ) -> WorthQueryWorkflowHistoryReconstructionBudget {
        self.history_reconstruction
    }
}

/// Separate finite admission for one retained-history reconstruction.
/// Transition visits bound traversal/replay work; bytes are a conservative
/// logical reservation for owned observations and progress, not allocator RSS.
/// A positive ceiling admits no minimum workload: if even the reader's fixed
/// scratch does not fit, reconstruction denies before allocating it.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorthQueryWorkflowHistoryReconstructionBudget {
    maximum_transition_visits: u32,
    maximum_charge_bytes: u64,
}

impl WorthQueryWorkflowHistoryReconstructionBudget {
    pub const fn new(maximum_transition_visits: u32, maximum_charge_bytes: u64) -> Option<Self> {
        if maximum_transition_visits == 0 || maximum_charge_bytes == 0 {
            return None;
        }
        Some(Self {
            maximum_transition_visits,
            maximum_charge_bytes,
        })
    }

    pub const fn standard() -> Self {
        Self {
            maximum_transition_visits: 16_384,
            maximum_charge_bytes: 128 * 1024 * 1024,
        }
    }

    pub const fn maximum_transition_visits(self) -> u32 {
        self.maximum_transition_visits
    }
    pub const fn maximum_charge_bytes(self) -> u64 {
        self.maximum_charge_bytes
    }
}
