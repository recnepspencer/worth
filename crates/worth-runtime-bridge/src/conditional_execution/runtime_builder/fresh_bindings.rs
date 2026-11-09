//! Installed programme bindings survive the consuming authored-owner handoff.
use super::BridgeConditionalRuntimeBuilder;
use crate::conditional_execution::{
    BridgeAuthoredDependencyDescription, BridgeAuthoredReadmissionDenial as D,
    BridgeConditionalContract, BridgeConditionalLocation, BridgeConditionalProviderSet,
    BridgeInstalledConditionalLowering,
};
use crate::correspondence::BridgeSemanticDependencyCandidate;
use std::sync::Arc;
use worth_execution::{ExecutionMemoryReservation, ExecutionRequest};

/// Live installation material, never decoded from the durable image.
pub struct BridgeFreshAuthoredBindings {
    lowerings: Vec<Arc<BridgeInstalledConditionalLowering>>,
    installation: worth_proof::BridgeProgramInstallationBinding,
    _memory: Option<ExecutionMemoryReservation>,
}
impl BridgeConditionalRuntimeBuilder {
    pub fn retain_fresh_authored_bindings(
        &self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<BridgeFreshAuthoredBindings, D> {
        let lowerings = self
            .runtime
            .conditional_lowerings
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let bytes = lowerings
            .len()
            .checked_mul(std::mem::size_of::<Arc<BridgeInstalledConditionalLowering>>())
            .and_then(|n| u64::try_from(n).ok())
            .ok_or(D::FreshBindingUnavailable)?;
        let memory = request.reserve_retained_memory(bytes).map_err(|cause| {
            D::Section(Box::new(
                crate::conditional_execution::BridgeAuthoredSectionDenial::Request(cause),
            ))
        })?;
        Ok(BridgeFreshAuthoredBindings {
            lowerings: lowerings.values().cloned().collect(),
            installation: self.program_owner.binding(),
            _memory: memory,
        })
    }
}
impl BridgeFreshAuthoredBindings {
    pub fn installation_binding(&self) -> worth_proof::BridgeProgramInstallationBinding {
        self.installation.clone()
    }
    /// Retain declaration selection only; this grants no execution of the revoked fresh installation.
    pub fn declaration(
        &self,
        lowering: &Arc<BridgeInstalledConditionalLowering>,
    ) -> Result<crate::conditional_execution::BridgeAuthoredConditionalDeclaration, D> {
        if !self
            .lowerings
            .iter()
            .any(|installed| Arc::ptr_eq(installed, lowering))
        {
            return Err(D::FreshBindingUnavailable);
        }
        Ok(
            crate::conditional_execution::BridgeAuthoredConditionalDeclaration {
                installation: self.installation.clone(),
                _installation: Arc::clone(lowering),
                key: crate::conditional_execution::authored_declaration::DeclarationKey::of(
                    lowering,
                ),
            },
        )
    }

    pub fn dependency(
        &self,
        description: &BridgeAuthoredDependencyDescription,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<BridgeSemanticDependencyCandidate, D> {
        let mut selected: Option<&BridgeSemanticDependencyCandidate> = None;
        for candidate in self
            .lowerings
            .iter()
            .flat_map(|lowering| lowering.semantic_dependencies())
        {
            if description.admits_fresh_candidate(request, candidate)? {
                if let Some(previous) = selected {
                    if previous.canonical_registration_key()
                        != candidate.canonical_registration_key()
                    {
                        return Err(D::FreshBindingUnavailable);
                    }
                } else {
                    selected = Some(candidate)
                }
            }
        }
        selected.cloned().ok_or(D::FreshBindingUnavailable)
    }
    pub fn providers(
        &self,
        contract: &BridgeConditionalContract,
        location: &BridgeConditionalLocation,
        descriptions: &crate::conditional_execution::BridgeAuthoredProviderDescriptions,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<BridgeConditionalProviderSet, D> {
        let mut matches = self.lowerings.iter().filter(|lowering| {
            lowering.contract().identity() == contract.identity()
                && lowering.contract().dependency_count() == contract.dependency_count()
                && lowering.location() == location
        });
        let selected = matches.next().ok_or(D::FreshBindingUnavailable)?;
        if matches.next().is_some() {
            return Err(D::FreshBindingUnavailable);
        }
        let providers = selected
            .providers
            .reconstruct_authored(descriptions, execution)
            .map_err(D::Provider)?;
        crate::conditional_execution::provider_admission::admit_provider_roles(
            contract, &providers,
        )
        .map_err(|cause| D::Conditional(Box::new(cause)))?;
        Ok(providers)
    }
}
