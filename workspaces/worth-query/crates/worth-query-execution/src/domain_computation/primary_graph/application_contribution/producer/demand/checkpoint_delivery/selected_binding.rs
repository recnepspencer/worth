//! Selected delivery borrows the actual installed route or the court's actual memory lowering.
use crate::domain_computation::primary_graph::{
    conditional_operation::installation::route_binding::InstalledConditionalRoute,
    managed_bridge::conditional_execution::ConditionalExecutionOwner,
};
use std::sync::Arc;
use worth_runtime_bridge::facade::{
    BridgeConditionalDenial, BridgeConditionalSignalBasisBinding,
    BridgeInstalledConditionalLowering,
};
#[derive(Clone, Copy)]
pub(super) enum SelectedDeliveryBinding<'a> {
    Installed(&'a InstalledConditionalRoute),
    Memory(&'a Arc<BridgeInstalledConditionalLowering>),
}
impl SelectedDeliveryBinding<'_> {
    pub(super) fn admit_signal_basis(
        &self,
        owner: &ConditionalExecutionOwner<'_>,
        basis: &worth_signal::facade::branch::AdmittedSignalBranchBasis,
    ) -> Result<BridgeConditionalSignalBasisBinding, BridgeConditionalDenial> {
        match self {
            Self::Installed(route) => route.admit_signal_basis(owner, basis),
            Self::Memory(lowering) => owner.admit_memory_signal_basis(lowering, basis),
        }
    }
}
