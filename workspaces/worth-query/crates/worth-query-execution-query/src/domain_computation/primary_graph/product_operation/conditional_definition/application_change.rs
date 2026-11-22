use std::sync::Arc;
use worth_query_execution_attempt::facade::ports::AdmittedApplicationConditionalDefinitionInternalPort as _;
use worth_query_execution_runtime::facade::ports::ProductBranchLeaseInternalPort as _;
use worth_query_execution_runtime::facade::ports::ProductRuntimeInternalPort as _;

pub use worth_query_execution_attempt::facade::domain_computation::primary_graph::product_operation::conditional_definition::application_change::{
    AdmittedSuccessor, WorthQueryAdmittedApplicationConditionalDefinition,
    WorthQueryApplicationConditionalDefinitionParts,
    WorthQueryCheckedApplicationConditionalDefinition,
};

pub trait WorthQueryAdmittedApplicationConditionalDefinitionRuntimeExt {
    fn new<Schema: worth_query_installation::facade::ApplicationSchema>(
        application: &crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        product: &crate::basis::WorthQueryProductBranchLease,
        identity: [u8; 32],
        successor: AdmittedSuccessor,
    ) -> Self;
}

impl WorthQueryAdmittedApplicationConditionalDefinitionRuntimeExt
    for WorthQueryAdmittedApplicationConditionalDefinition
{
    fn new<Schema: worth_query_installation::facade::ApplicationSchema>(
        application: &crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        product: &crate::basis::WorthQueryProductBranchLease,
        identity: [u8; 32],
        successor: AdmittedSuccessor,
    ) -> Self {
        Self::from_admitted_parts(
            application.runtime.authority_identity(),
            application.installed_schema().binding_identity(),
            product.observation().clone(),
            identity,
            successor,
            application.runtime.bridge.conditional_operations(),
            Arc::clone(&application.runtime.product_runtime.activations_ref()),
        )
    }
}
