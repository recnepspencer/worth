use std::sync::Arc;

use worth_runtime_bridge::facade::{
    BridgeInstalledConditionalLowering, BridgeOwnedConditionalInstallationRequest,
};

pub use worth_query_execution_attempt::facade::domain_computation::primary_graph::product_operation::conditional_definition::application_change::{
    WorthQueryAdmittedApplicationConditionalDefinition,
    WorthQueryApplicationConditionalDefinitionParts,
};

pub trait WorthQueryAdmittedApplicationConditionalDefinitionRuntimeExt {
    fn new<Schema: worth_query_installation::facade::ApplicationSchema>(
        application: &crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        product: &crate::basis::WorthQueryProductBranchLease,
        identity: [u8; 32],
        predecessor: Arc<BridgeInstalledConditionalLowering>,
        request: BridgeOwnedConditionalInstallationRequest,
    ) -> Self;
}

impl WorthQueryAdmittedApplicationConditionalDefinitionRuntimeExt
    for WorthQueryAdmittedApplicationConditionalDefinition
{
    fn new<Schema: worth_query_installation::facade::ApplicationSchema>(
        application: &crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        product: &crate::basis::WorthQueryProductBranchLease,
        identity: [u8; 32],
        predecessor: Arc<BridgeInstalledConditionalLowering>,
        request: BridgeOwnedConditionalInstallationRequest,
    ) -> Self {
        Self::from_admitted_parts(
            application.runtime.authority_identity(),
            application.installed_schema().binding_identity(),
            product.observation().clone(),
            identity,
            predecessor,
            request,
            application.runtime.bridge.conditional_operations(),
            Arc::clone(&application.runtime.product_runtime.activations),
        )
    }
}
