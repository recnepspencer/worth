//! Inherited live bindings come from the exact source projection, never bytes.
use super::*;
use crate::conditional_execution::{
    BridgeAuthoredInstallationBindings, BridgeConditionalContract, BridgeConditionalLocation,
    BridgeConditionalProviderSet,
};
use crate::correspondence::BridgeSemanticDependencyCandidate;
use worth_foundational::facade::BridgeSemanticClaimReference;

pub(super) struct InheritedBindings<'a> {
    pub(super) source: Arc<BranchProjection>,
    pub(super) inherited: &'a [Arc<BridgeInstalledConditionalLowering>],
}
impl BridgeAuthoredInstallationBindings for InheritedBindings<'_> {
    fn dependency(
        &mut self,
        _: DurableSignalBranchIdentity,
        reference: BridgeSemanticClaimReference,
        _: &crate::conditional_execution::BridgeAuthoredDependencyDescription,
        _: &ExecutionRequest<'_, '_>,
    ) -> Result<BridgeSemanticDependencyCandidate, BridgeAuthoredReadmissionDenial> {
        let source = self
            .source
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        source
            ._dependencies
            .admitted_entries()
            .find_map(|(_, registration)| {
                (registration.recorded_claim == Some(reference))
                    .then(|| registration.dependency().clone())
            })
            .ok_or(BridgeAuthoredReadmissionDenial::FreshBindingUnavailable)
    }
    fn providers(
        &mut self,
        _: DurableSignalBranchIdentity,
        contract: &BridgeConditionalContract,
        location: &BridgeConditionalLocation,
        _: &ExecutionRequest<'_, '_>,
    ) -> Result<BridgeConditionalProviderSet, BridgeAuthoredReadmissionDenial> {
        self.inherited
            .iter()
            .find(|lowering| lowering.contract() == contract && lowering.location() == location)
            .map(|lowering| lowering.providers.clone())
            .ok_or(BridgeAuthoredReadmissionDenial::FreshBindingUnavailable)
    }
}
