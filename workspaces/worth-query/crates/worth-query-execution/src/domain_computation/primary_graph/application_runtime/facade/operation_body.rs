//! The runtime's consumer methods over `domain_computation::primary_graph::operation_body`; each only calls its subsystem.
use crate::domain_computation::primary_graph::operation_body::WorthQueryOperationBodyRequirementFinding;
use crate::domain_computation::primary_graph::WorthQueryAdmittedApplicationOperation;
use crate::domain_computation::primary_graph::WorthQueryAdvancementPhase;
use crate::domain_computation::primary_graph::WorthQueryApplicationInvariantProjectionAuthority;
use crate::domain_computation::primary_graph::WorthQueryCurrentBodyRequirementAssessment;
use crate::domain_computation::primary_graph::WorthQueryOperationBodyAssessmentDenial;
use worth_query_declaration::facade::application_operation::ApplicationMutationBinding;
use worth_query_declaration::facade::application_operation::ApplicationMutationScopeBinding;
use worth_query_installation::facade::ApplicationSchema;
use worth_query_installation::facade::WorthQueryInstalledApplicationMutationBinding;
impl<Schema: ApplicationSchema> super::super::WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    pub fn assess_admitted_operation_body_requirements<Binding>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        admission: &WorthQueryAdmittedApplicationOperation<
            Schema,
            Binding::Operation,
            Binding::Input,
            <Binding::ScopeBinding as ApplicationMutationScopeBinding<Schema>>::Scope,
        >,
        installed: &WorthQueryInstalledApplicationMutationBinding<Schema, Binding>,
        projection: &WorthQueryApplicationInvariantProjectionAuthority<Schema>,
        allocation: worth_execution::ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<WorthQueryCurrentBodyRequirementAssessment, WorthQueryOperationBodyAssessmentDenial>
    where
        Binding: ApplicationMutationBinding<Schema>,
    {
        <Self as crate::domain_computation::primary_graph::operation_body::AssessmentOps<Schema>>::assess_admitted_operation_body_requirements(self, phase, admission, installed, projection, allocation)
    }
}
