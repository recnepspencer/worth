use std::sync::Arc;

use worth_runtime_bridge::facade::{
    BridgeInstalledConditionalLowering, BridgeOwnedConditionalInstallationRequest,
};
use worth_runtime_world::facade::ProductBranchObservation;

/// A concrete conditional-definition successor admitted by the selected
/// application and product occurrence. The value is move-only and can enter
/// exactly one application effect program.
pub struct WorthQueryAdmittedApplicationConditionalDefinition {
    runtime_authority:
        crate::domain_computation::execution_runtime::WorthQueryRuntimeAuthorityIdentity,
    application_binding: worth_query_installation::facade::ApplicationSchemaBindingIdentity,
    expected_product: ProductBranchObservation,
    identity: [u8; 32],
    successor: AdmittedSuccessor,
    bridge: crate::domain_computation::primary_graph::managed_bridge::conditional_owner::ConditionalOwner,
    activations: Arc<
        crate::domain_computation::execution_runtime::product_world::activation::WorthQueryProductActivationRegistry,
    >,
}

/// The Bridge successor at the selected product's Signal basis, or why that
/// basis serves no installed definition. A refusal is reported by the commit
/// only after its idempotency key resolves, so a retained original answers
/// its key before any Bridge basis check can refuse it.
type AdmittedSuccessor = Result<
    (
        Arc<BridgeInstalledConditionalLowering>,
        BridgeOwnedConditionalInstallationRequest,
    ),
    worth_runtime_bridge::facade::BridgeConditionalDenial,
>;

pub(in crate::domain_computation::primary_graph) struct
    WorthQueryApplicationConditionalDefinitionParts
{
    pub expected_product: ProductBranchObservation,
    pub predecessor: Arc<BridgeInstalledConditionalLowering>,
    pub request: BridgeOwnedConditionalInstallationRequest,
    pub bridge: crate::domain_computation::primary_graph::managed_bridge::conditional_owner::ConditionalOwner,
    pub activations: Arc<
        crate::domain_computation::execution_runtime::product_world::activation::WorthQueryProductActivationRegistry,
    >,
}

impl WorthQueryAdmittedApplicationConditionalDefinition {
    pub(super) fn new<Schema: worth_query_installation::facade::ApplicationSchema>(
        application: &crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        product: &crate::basis::WorthQueryProductBranchLease,
        identity: [u8; 32],
        successor: AdmittedSuccessor,
    ) -> Self {
        Self {
            runtime_authority: application.runtime.authority_identity(),
            application_binding: application.installed_schema().binding_identity(),
            expected_product: product.observation().clone(),
            identity,
            successor,
            bridge: application.runtime.bridge.conditional_operations(),
            activations: Arc::clone(&application.runtime.product_runtime.activations),
        }
    }

    pub(in crate::domain_computation::primary_graph) fn admits_application(
        &self,
        runtime_authority: crate::domain_computation::execution_runtime::WorthQueryRuntimeAuthorityIdentity,
        application_binding: &worth_query_installation::facade::ApplicationSchemaBindingIdentity,
        product: &ProductBranchObservation,
    ) -> bool {
        self.runtime_authority == runtime_authority
            && &self.application_binding == application_binding
            && &self.expected_product == product
    }

    pub(in crate::domain_computation::primary_graph) const fn identity(&self) -> &[u8; 32] {
        &self.identity
    }

    /// Why the selected product's Signal basis serves no installed
    /// predecessor, checked by the commit after its key resolves.
    pub(in crate::domain_computation::primary_graph) fn successor_refusal(
        &self,
    ) -> Option<&worth_runtime_bridge::facade::BridgeConditionalDenial> {
        self.successor.as_ref().err()
    }

    pub(in crate::domain_computation::primary_graph) fn into_parts(
        self,
    ) -> Result<
        WorthQueryApplicationConditionalDefinitionParts,
        worth_runtime_bridge::facade::BridgeConditionalDenial,
    > {
        let (predecessor, request) = self.successor?;
        Ok(WorthQueryApplicationConditionalDefinitionParts {
            expected_product: self.expected_product,
            predecessor,
            request,
            bridge: self.bridge,
            activations: self.activations,
        })
    }
}

impl std::fmt::Debug for WorthQueryAdmittedApplicationConditionalDefinition {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("WorthQueryAdmittedApplicationConditionalDefinition")
            .field("expected_product", &self.expected_product)
            .field("identity", &self.identity)
            .finish_non_exhaustive()
    }
}
