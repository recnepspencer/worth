//! A Query route carries installed memory authority or an authored declaration, exhaustively.
use crate::domain_computation::primary_graph::managed_bridge::conditional_execution::ConditionalExecutionOwner;
use std::sync::Arc;
use worth_runtime_bridge::facade::{
    BridgeAuthoredConditionalDeclaration, BridgeAuthoredReadmissionDenial, BridgeConditionalDenial,
    BridgeConditionalSignalBasisBinding, BridgeFreshAuthoredBindings,
    BridgeInstalledConditionalLowering,
};

#[derive(Clone)]
pub(in crate::domain_computation::primary_graph) enum InstalledConditionalRoute {
    Memory(Arc<BridgeInstalledConditionalLowering>),
    Authored(BridgeAuthoredConditionalDeclaration),
}
impl InstalledConditionalRoute {
    pub(in crate::domain_computation::primary_graph) fn from_installation(
        lowering: Arc<BridgeInstalledConditionalLowering>,
    ) -> Self {
        Self::Memory(lowering)
    }
    pub(in crate::domain_computation::primary_graph) fn admit_authored_declaration(
        &mut self,
        fresh: &BridgeFreshAuthoredBindings,
    ) -> Result<(), BridgeAuthoredReadmissionDenial> {
        match self {
            Self::Memory(lowering) => {
                let declaration = fresh.declaration(lowering)?;
                *self = Self::Authored(declaration);
                Ok(())
            }
            Self::Authored(_) => Ok(()),
        }
    }
    pub(in crate::domain_computation::primary_graph) fn admit_signal_basis(
        &self,
        owner: &ConditionalExecutionOwner<'_>,
        basis: &worth_signal::facade::branch::AdmittedSignalBranchBasis,
    ) -> Result<BridgeConditionalSignalBasisBinding, BridgeConditionalDenial> {
        match self {
            Self::Memory(lowering) => owner.admit_memory_signal_basis(lowering, basis),
            Self::Authored(declaration) => owner.admit_declaration_signal_basis(declaration, basis),
        }
    }
}
