//! Thin forwarding shims for `application_entry` methods onto `worth-query-execution-query`.

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_application_entry_mutation_execution {
    use crate::domain_computation::primary_graph::WorthQueryAdvancementPhase;
    use worth_query_declaration::facade::application_operation::ApplicationMutationBinding;
    use worth_query_declaration::facade::application_operation::ApplicationMutationIdentities;
    use worth_query_declaration::facade::application_operation::ApplicationMutationScopeBinding;
    use worth_query_installation::facade::ApplicationSchema;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_entry::mutation::HandlerResult;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_entry::mutation::WorthQueryCompletedMutationCandidate;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_entry::mutation::WorthQueryMutationHandlerExecutionReport;
    use crate::domain_computation::primary_graph::WorthQueryAdmittedApplicationOperation;
    use worth_query_execution_lineage::facade::primary_graph::application_entry::mutation::execution::MutationHandlerExecutionDenial;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
    where
        Schema: ApplicationSchema,
    {
        pub fn execute_mutation_handler<Binding>(
            &self,
            phase: &WorthQueryAdvancementPhase<'_>,
            identities: &ApplicationMutationIdentities<'_, Schema, Binding>,
            principal_identity: &Binding::PrincipalIdentity,
            admission: WorthQueryAdmittedApplicationOperation<
                Schema,
                Binding::Operation,
                Binding::Input,
                <Binding::ScopeBinding as ApplicationMutationScopeBinding<Schema>>::Scope,
            >,
            allocation_policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
        ) -> Result<
            HandlerResult<WorthQueryCompletedMutationCandidate<Schema, Binding>, Binding::Denial>,
            MutationHandlerExecutionDenial,
        >
        where
            Binding: ApplicationMutationBinding<Schema>,
        {
            self.inner.execute_mutation_handler::<Binding>(
                phase,
                identities,
                principal_identity,
                admission,
                allocation_policy,
            )
        }
        pub fn execute_mutation_handler_report<Binding>(
            &self,
            phase: &WorthQueryAdvancementPhase<'_>,
            identities: &ApplicationMutationIdentities<'_, Schema, Binding>,
            principal_identity: &Binding::PrincipalIdentity,
            admission: WorthQueryAdmittedApplicationOperation<
                Schema,
                Binding::Operation,
                Binding::Input,
                <Binding::ScopeBinding as ApplicationMutationScopeBinding<Schema>>::Scope,
            >,
            allocation_policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
        ) -> WorthQueryMutationHandlerExecutionReport<
            WorthQueryCompletedMutationCandidate<Schema, Binding>,
            Binding::Denial,
        >
        where
            Binding: ApplicationMutationBinding<Schema>,
        {
            self.inner.execute_mutation_handler_report::<Binding>(
                phase,
                identities,
                principal_identity,
                admission,
                allocation_policy,
            )
        }
        pub fn execute_mutation_handler_report_observing_contact<Binding>(
            &self,
            phase: &WorthQueryAdvancementPhase<'_>,
            identities: &ApplicationMutationIdentities<'_, Schema, Binding>,
            principal_identity: &Binding::PrincipalIdentity,
            admission: WorthQueryAdmittedApplicationOperation<
                Schema,
                Binding::Operation,
                Binding::Input,
                <Binding::ScopeBinding as ApplicationMutationScopeBinding<Schema>>::Scope,
            >,
            on_contact: impl FnOnce(),
            allocation_policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
        ) -> WorthQueryMutationHandlerExecutionReport<
            WorthQueryCompletedMutationCandidate<Schema, Binding>,
            Binding::Denial,
        >
        where
            Binding: ApplicationMutationBinding<Schema>,
        {
            self.inner
                .execute_mutation_handler_report_observing_contact::<Binding>(
                    phase,
                    identities,
                    principal_identity,
                    admission,
                    on_contact,
                    allocation_policy,
                )
        }
    }
}

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_application_entry_mutation_execution_decision_query_runtime {
    use worth_query_admission::facade::authenticated_principal::WorthQueryAuthenticatedExternalPrincipal;
    use crate::domain_computation::primary_graph::WorthQueryAdvancementPhase;
    use worth_query_declaration::facade::application_operation::ApplicationMutationBinding;
    use worth_query_declaration::facade::application_operation::ApplicationMutationIdentities;
    use worth_query_declaration::facade::application_operation::ApplicationMutationScopeBinding;
    use worth_query_installation::facade::ApplicationSchema;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_entry::mutation::WorthQueryCompletedMutationCandidate;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_entry::mutation::WorthQueryMutationHandlerExecutionReport;
    use crate::domain_computation::primary_graph::WorthQueryAdmittedApplicationOperation;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    impl<S: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<S> {
        pub fn execute_mutation_handler_report_with_query_reads<
            B: ApplicationMutationBinding<S>,
        >(
            &self,
            phase: &WorthQueryAdvancementPhase<'_>,
            identities: &ApplicationMutationIdentities<'_, S, B>,
            principal_identity: &B::PrincipalIdentity,
            principal: &WorthQueryAuthenticatedExternalPrincipal<S>,
            admission: WorthQueryAdmittedApplicationOperation<
                S,
                B::Operation,
                B::Input,
                <B::ScopeBinding as ApplicationMutationScopeBinding<S>>::Scope,
            >,
            allocation: worth_execution::ExecutionAllocationPolicy<'_, '_>,
        ) -> WorthQueryMutationHandlerExecutionReport<
            WorthQueryCompletedMutationCandidate<S, B>,
            B::Denial,
        > {
            self.inner
                .execute_mutation_handler_report_with_query_reads::<B>(
                    phase,
                    identities,
                    principal_identity,
                    principal,
                    admission,
                    allocation,
                )
        }
    }
}
