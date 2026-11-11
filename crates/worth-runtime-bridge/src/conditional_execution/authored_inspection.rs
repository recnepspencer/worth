//! Weak observations come from every actual recovered branch and its derived registry.
use super::{authored_section::branch_registry::BranchSlot, BridgeReadmittedAuthoredAssembly};
use std::sync::Arc;
impl BridgeReadmittedAuthoredAssembly {
    pub fn conditional_lifecycle_probe(&self) -> super::BridgeConditionalRuntimeLifecycleProbe {
        let mut providers = Vec::new();
        let mut retention = Vec::new();
        let mut signal_graphs = Vec::new();
        let slots = self
            .branches
            .slots
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        for slot in slots.values() {
            match slot {
                BranchSlot::Serving { projection, .. } => {
                    let state = projection
                        .state
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner);
                    providers.extend(
                        state
                            .lowerings
                            .read()
                            .unwrap_or_else(std::sync::PoisonError::into_inner)
                            .values()
                            .map(Arc::downgrade),
                    );
                    retention.push(Arc::downgrade(&state.retention));
                    signal_graphs.push(state.graph_lifecycle.clone());
                }
                BranchSlot::Reserved { .. } | BranchSlot::ReadmissionRequired(_) => {}
            }
        }
        drop(slots);
        super::BridgeConditionalRuntimeLifecycleProbe {
            signal_graphs,
            conditional_port: None,
            providers,
            managed_clocks: Vec::new(),
            retention,
        }
    }
    pub fn installed_conditional_definition_count(&self) -> usize {
        self.conditional_lifecycle_probe().live_provider_count()
    }
    pub fn installed_conditional_target_reference_count(&self) -> usize {
        let slots = self
            .branches
            .slots
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let mut count = 0;
        for slot in slots.values() {
            match slot {
                BranchSlot::Serving { projection, .. } => {
                    let state = projection
                        .state
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner);
                    for lowering in state
                        .lowerings
                        .read()
                        .unwrap_or_else(std::sync::PoisonError::into_inner)
                        .values()
                    {
                        count += lowering
                            .correspondences
                            .iter()
                            .map(|item| item.targets.as_slice().len())
                            .sum::<usize>();
                    }
                }
                BranchSlot::Reserved { .. } | BranchSlot::ReadmissionRequired(_) => {}
            }
        }
        count
    }
    pub fn owned_signal_active_node_count(&self) -> Result<usize, super::BridgeConditionalDenial> {
        self.signal
            .source_port()
            .topology_observations()
            .map(|items| items.into_iter().map(|(count, _)| count).sum())
            .map_err(|cause| {
                super::BridgeConditionalDenial::new(
                    super::BridgeConditionalDenialKind::StaleLowering,
                    format!("Signal topology observation refused: {cause:?}"),
                )
            })
    }
    #[doc(hidden)]
    pub fn signal_operation_control_for_test(
        &self,
    ) -> worth_signal::facade::branch::SignalOwnerOperationControlForTest {
        self.signal.signal_operation_control_for_test()
    }
    #[doc(hidden)]
    pub fn configure_conditional_history_for_test(&self, enabled: bool, limit: usize) -> bool {
        self.program
            .bridge
            .diagnostics
            .configure_conditional_history_for_test(enabled, limit)
    }
}
