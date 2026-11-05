//! The runtime composes `product_operation`.
use worth_query_installation::facade::ApplicationSchema;

use super::super::WorthQueryPrimaryGraphApplicationRuntime;
use crate::domain_computation::primary_graph::product_operation::host::ProductOperationHost;

impl<Schema: ApplicationSchema> ProductOperationHost<Schema>
    for WorthQueryPrimaryGraphApplicationRuntime<Schema>
{
    fn product_operation_host(&self) -> &dyn ProductOperationHost<Schema> {
        self
    }
    fn conditional_operations(&self) -> &std::sync::Mutex<crate::domain_computation::primary_graph::conditional_operation::WorthQueryConditionalOperationRegistry<Schema>>{
        &self.conditional_operations
    }
    fn application_readiness_schema_token(&self) -> &str {
        &self.runtime.application_readiness_schema_token
    }
    fn basis_leases(&self) -> &crate::domain_computation::primary_graph::application_query::resource_lifecycle::WorthQueryApplicationBasisRegistry{
        &self.runtime.basis_leases
    }
    fn installed_producers(&self) -> &crate::domain_computation::primary_graph::application_contribution::WorthQueryInstalledApplicationProducerRegistry<Schema>{
        &self.output_production.producers()
    }
    fn workflow_coverage(&self) -> &crate::domain_computation::primary_graph::application_attempt::workflow::adoption::WorkflowVocabularyCoverageRegistry{
        &self.runtime.workflow_coverage
    }
}

impl<Schema: ApplicationSchema> crate::basis::WorthQuerySourceProgramResolver
    for WorthQueryPrimaryGraphApplicationRuntime<Schema>
{
    fn source_program(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        source: &crate::basis::WorthQueryProductBranchLease,
    ) -> Result<
        Option<worth_query_declaration::facade::application_program::ApplicationProgramRevision>,
        crate::basis::WorthQueryProductBranchAdmissionDenial,
    > {
        match crate::domain_computation::primary_graph::product_activation::inspect_selected_program(execution, self, source.relational_basis()) {
            Ok(selected) => Ok(Some(*selected.revision())),
            Err(crate::domain_computation::primary_graph::vocabulary::product_activation::inspection::WorthQuerySelectedProgramInspectionDenial::ReadExecution(stop)) => Err(crate::basis::WorthQueryProductBranchAdmissionDenial::Stopped(stop)),
            Err(crate::domain_computation::primary_graph::vocabulary::product_activation::inspection::WorthQuerySelectedProgramInspectionDenial::ProgramSupportUnavailable | crate::domain_computation::primary_graph::vocabulary::product_activation::inspection::WorthQuerySelectedProgramInspectionDenial::ProgramActivationUnavailable | crate::domain_computation::primary_graph::vocabulary::product_activation::inspection::WorthQuerySelectedProgramInspectionDenial::ProgramActivationUnreadable | crate::domain_computation::primary_graph::vocabulary::product_activation::inspection::WorthQuerySelectedProgramInspectionDenial::ProgramActivationUnrostered) => Ok(None),
        }
    }
}
