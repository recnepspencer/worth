//! Thin forwarding shims for `operation_body` methods onto `worth-query-execution-query`.

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_operation_body_runtime_assessment {
    use worth_query_declaration::facade::application_operation::ApplicationMutationBinding;
    use worth_query_declaration::facade::application_operation::ApplicationMutationScopeBinding;
    use worth_query_declaration::facade::application_schema::ApplicationSchema;
    use worth_query_installation::facade::WorthQueryInstalledApplicationMutationBinding;
    use crate::domain_computation::primary_graph::WorthQueryAdmittedApplicationOperation;
    use crate::domain_computation::primary_graph::WorthQueryAdvancementPhase;
    use crate::domain_computation::primary_graph::WorthQueryApplicationInvariantProjectionAuthority;
    use crate::domain_computation::primary_graph::WorthQueryCurrentBodyRequirementAssessment;
    use crate::domain_computation::primary_graph::WorthQueryOperationBodyAssessmentDenial;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
        pub fn assess_admitted_operation_body_requirements<Binding>(&self, phase: &WorthQueryAdvancementPhase<'_>, admission: &WorthQueryAdmittedApplicationOperation<
                Schema,
                Binding::Operation,
                Binding::Input,
                <Binding::ScopeBinding as ApplicationMutationScopeBinding<Schema>>::Scope,
            >, installed: &WorthQueryInstalledApplicationMutationBinding<Schema, Binding>, projection: &WorthQueryApplicationInvariantProjectionAuthority<Schema>, allocation: worth_execution::ExecutionAllocationPolicy<'_, '_>) -> Result<WorthQueryCurrentBodyRequirementAssessment, WorthQueryOperationBodyAssessmentDenial>
        where
            Binding: ApplicationMutationBinding<Schema>, {
            self.inner.assess_admitted_operation_body_requirements::<Binding>(phase, admission, installed, projection, allocation)
        }
    }
}
