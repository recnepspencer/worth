//! Query retains bindings derived by its actual fresh conditional installation.
use worth_execution::ExecutionRequest;
use worth_foundational::facade::{BridgeSemanticClaimReference, DurableSignalBranchIdentity};
use worth_runtime_bridge::facade::*;

pub struct QueryHomeAuthoredBindings {
    fresh: BridgeFreshAuthoredBindings,
}
impl QueryHomeAuthoredBindings {
    pub(super) fn installation_binding(&self) -> worth_proof::BridgeProgramInstallationBinding {
        self.fresh.installation_binding()
    }
    pub(super) fn admit_routes(
        &self,
        producers: &mut crate::domain_computation::primary_graph::application_contribution::WorthQueryInstalledOutputProducerRoutes,
        readiness: &mut crate::domain_computation::primary_graph::application_contribution::WorthQueryInstalledOutputReadinessRoutes,
        definitions: &mut crate::domain_computation::primary_graph::conditional_operation::definition_declarations::ConditionalDefinitionDeclarations,
    ) -> Result<(), BridgeAuthoredReadmissionDenial> {
        for route in producers.values_mut() {
            route.scheduling.admit_authored_declaration(&self.fresh)?;
            if let Some(comparison) = &mut route.output_comparison {
                comparison.admit_authored_declaration(&self.fresh)?;
            }
        }
        definitions.admit_authored_declarations(&self.fresh)?;
        for route in readiness.values_mut() {
            route.lowering.admit_authored_declaration(&self.fresh)?;
        }
        Ok(())
    }

    pub(super) fn from_installed_program(
        builder: &BridgeConditionalRuntimeBuilder,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<Self, BridgeAuthoredReadmissionDenial> {
        Ok(Self {
            fresh: builder.retain_fresh_authored_bindings(execution)?,
        })
    }
}
impl BridgeAuthoredInstallationBindings for QueryHomeAuthoredBindings {
    fn dependency(
        &mut self,
        _branch: DurableSignalBranchIdentity,
        _reference: BridgeSemanticClaimReference,
        description: &BridgeAuthoredDependencyDescription,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<BridgeSemanticDependencyCandidate, BridgeAuthoredReadmissionDenial> {
        self.fresh.dependency(description, execution)
    }
    fn providers(
        &mut self,
        _branch: DurableSignalBranchIdentity,
        contract: &BridgeConditionalContract,
        location: &BridgeConditionalLocation,
        descriptions: &BridgeAuthoredProviderDescriptions,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<BridgeConditionalProviderSet, BridgeAuthoredReadmissionDenial> {
        execution.consult().map_err(|cause| {
            BridgeAuthoredReadmissionDenial::Section(Box::new(
                BridgeAuthoredSectionDenial::Request(cause),
            ))
        })?;
        self.fresh
            .providers(contract, location, descriptions, execution)
    }
}
