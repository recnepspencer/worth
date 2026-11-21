//! Thin forwarding shims for `workflow` methods onto `worth-query-execution-query`.

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_workflow_captured_input {
    use worth_query_execution_attempt::facade::domain_computation::primary_graph::workflow::captured_input::*;
    use worth_query_declaration::facade::application_operation::ApplicationMutationBinding;
    use worth_query_declaration::facade::application_operation::ApplicationMutationIdentities;
    use worth_query_declaration::facade::application_operation::ApplicationMutationScopeBinding;
    use worth_query_declaration::facade::application_operation::ApplicationMutationSourceExpectation;
    use worth_query_declaration::facade::application_schema::TypedMutationPreconditions;
    use worth_query_installation::facade::ApplicationSchema;
    use crate::domain_computation::primary_graph::WorthQueryAdvancementPhase;
    use crate::domain_computation::primary_graph::WorthQueryApplicationEntityIdentity;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    type Scope<S, B> =
        <<B as ApplicationMutationBinding<S>>::ScopeBinding as ApplicationMutationScopeBinding<
            S,
        >>::Scope;
    type Query<S, B> = <<B as ApplicationMutationBinding<S>>::SourceExpectation as ApplicationMutationSourceExpectation<S>>::Query;
    impl<S: ApplicationSchema> WorthQuerySelectedProductOperation<'_, S> {
        pub fn prepare_workflow_captured_input<B>(
            &self,
            phase: &WorthQueryAdvancementPhase<'_>,
            __arg1: &crate::publication_boundary::WorthQueryProgramPublicationAccess,
            scope: &WorthQueryApplicationEntityIdentity<S, Scope<S, B>>,
            identities: &ApplicationMutationIdentities<'_, S, B>,
            preconditions: &TypedMutationPreconditions<S, B::Operation, Scope<S, B>>,
            source: WorkflowCapturedInputSource<'_, Query<S, B>>,
        ) -> Result<PreparedWorkflowCapturedInput, WorkflowCapturedInputDenial>
        where
            B: ApplicationMutationBinding<S>,
        {
            self.inner.prepare_workflow_captured_input::<B>(
                phase,
                __arg1,
                scope,
                identities,
                preconditions,
                source,
            )
        }
    }
}
