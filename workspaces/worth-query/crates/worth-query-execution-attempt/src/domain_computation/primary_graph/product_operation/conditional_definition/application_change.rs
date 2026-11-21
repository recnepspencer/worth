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
    predecessor: Arc<BridgeInstalledConditionalLowering>,
    request: BridgeOwnedConditionalInstallationRequest,
    bridge: crate::domain_computation::primary_graph::managed_bridge::conditional_owner::ConditionalOwner,
    activations: Arc<
        crate::domain_computation::execution_runtime::product_world::activation::WorthQueryProductActivationRegistry,
    >,
}

pub struct
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
    pub fn from_admitted_parts(
        runtime_authority: crate::domain_computation::execution_runtime::WorthQueryRuntimeAuthorityIdentity,
        application_binding: worth_query_installation::facade::ApplicationSchemaBindingIdentity,
        expected_product: ProductBranchObservation,
        identity: [u8; 32],
        predecessor: Arc<BridgeInstalledConditionalLowering>,
        request: BridgeOwnedConditionalInstallationRequest,
        bridge: crate::domain_computation::primary_graph::managed_bridge::conditional_owner::ConditionalOwner,
        activations: Arc<
            crate::domain_computation::execution_runtime::product_world::activation::WorthQueryProductActivationRegistry,
        >,
    ) -> Self {
        Self {
            runtime_authority,
            application_binding,
            expected_product,
            identity,
            predecessor,
            request,
            bridge,
            activations,
        }
    }

    pub fn admits_application(
        &self,
        runtime_authority: crate::domain_computation::execution_runtime::WorthQueryRuntimeAuthorityIdentity,
        application_binding: &worth_query_installation::facade::ApplicationSchemaBindingIdentity,
        product: &ProductBranchObservation,
    ) -> bool {
        self.runtime_authority == runtime_authority
            && &self.application_binding == application_binding
            && &self.expected_product == product
    }

    pub const fn identity(&self) -> &[u8; 32] {
        &self.identity
    }

    pub fn into_parts(self) -> WorthQueryApplicationConditionalDefinitionParts {
        WorthQueryApplicationConditionalDefinitionParts {
            expected_product: self.expected_product,
            predecessor: self.predecessor,
            request: self.request,
            bridge: self.bridge,
            activations: self.activations,
        }
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
