//! Thin forwarding shims for `application_query` methods onto `worth-query-execution-query`.

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_application_query_derived_view_registration_collection_pair_refresh_batch {
    use worth_query_declaration::facade::application_schema::ApplicationSchema;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_query::derived_view::registration::collection::pair::WorthQueryAdmittedApplicationQueryPlan;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_query::derived_view::registration::collection::pair::WorthQueryApplicationProjection;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_query::derived_view::registration::collection::pair::WorthQueryManagedDerivedValue;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_query::derived_view::registration::collection::pair::WorthQueryManagedDerivedView;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_query::derived_view::registration::collection::pair::WorthQueryManagedDerivedViewKey;
    use crate::basis::WorthQueryProductBranchLease;
    use crate::domain_computation::primary_graph::application_query::WorthQueryApplicationBatchResult;
    use crate::domain_computation::primary_graph::application_query::WorthQueryApplicationQueryBatchAdmission;
    use crate::domain_computation::primary_graph::application_query::WorthQueryManagedDerivedCollectionBatchRefreshDenial as Denial;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
        #[allow(clippy::too_many_arguments)]
        pub fn refresh_managed_derived_collection_pair_entry_in_batch<
            'a,
            MembershipQuery,
            FirstQuery,
            FirstParameters,
            FirstResult,
            FirstPrincipal,
            FirstPrincipalIdentity,
            FirstScope,
            SecondQuery,
            SecondResult,
            Value,
            LinkKey,
        >(
            &self,
            execution: &worth_execution::ExecutionRequest<'_, '_>,
            view: &WorthQueryManagedDerivedView<MembershipQuery, Value>,
            product: &WorthQueryProductBranchLease,
            key: &WorthQueryManagedDerivedViewKey,
            first: WorthQueryAdmittedApplicationQueryPlan<
                'a,
                Schema,
                FirstQuery,
                FirstParameters,
                FirstResult,
                FirstPrincipal,
                FirstPrincipalIdentity,
                FirstScope,
            >,
            batch: &WorthQueryApplicationQueryBatchAdmission,
            second_for: impl FnOnce(
                &FirstResult,
                &WorthQueryApplicationQueryBatchAdmission,
            ) -> Result<
                WorthQueryApplicationBatchResult<SecondQuery, SecondResult>,
                Denial,
            >,
            first_link: impl FnOnce(&FirstResult) -> LinkKey,
            second_link: impl FnOnce(&SecondResult) -> LinkKey,
            project: impl FnOnce(&FirstResult, &SecondResult) -> Value,
        ) -> Result<std::sync::Arc<Value>, Denial>
        where
            FirstQuery: 'a,
            FirstParameters: 'a,
            FirstResult: WorthQueryApplicationProjection<Schema, FirstQuery> + 'a,
            FirstPrincipal: 'a,
            FirstPrincipalIdentity: 'a,
            FirstScope: 'a,
            SecondResult: WorthQueryApplicationProjection<Schema, SecondQuery> + 'a,
            Value: WorthQueryManagedDerivedValue,
            LinkKey: Eq,
        {
            self.inner.refresh_managed_derived_collection_pair_entry_in_batch::<MembershipQuery, FirstQuery, FirstParameters, FirstResult, FirstPrincipal, FirstPrincipalIdentity, FirstScope, SecondQuery, SecondResult, Value, LinkKey>(execution, view, product, key, first, batch, second_for, first_link, second_link, project)
        }
        #[allow(clippy::too_many_arguments)]
        pub fn refresh_managed_derived_collection_pair_entry_from_batch_result<
            MembershipQuery,
            FirstQuery,
            FirstResult,
            SecondQuery,
            SecondResult,
            Value,
            LinkKey,
        >(
            &self,
            view: &WorthQueryManagedDerivedView<MembershipQuery, Value>,
            product: &WorthQueryProductBranchLease,
            key: &WorthQueryManagedDerivedViewKey,
            first: WorthQueryApplicationBatchResult<FirstQuery, FirstResult>,
            batch: &WorthQueryApplicationQueryBatchAdmission,
            second_for: impl FnOnce(
                &FirstResult,
                &WorthQueryApplicationQueryBatchAdmission,
            ) -> Result<
                WorthQueryApplicationBatchResult<SecondQuery, SecondResult>,
                Denial,
            >,
            first_link: impl FnOnce(&FirstResult) -> LinkKey,
            second_link: impl FnOnce(&SecondResult) -> LinkKey,
            project: impl FnOnce(&FirstResult, &SecondResult) -> Value,
        ) -> Result<std::sync::Arc<Value>, Denial>
        where
            FirstResult: WorthQueryApplicationProjection<Schema, FirstQuery>,
            SecondResult: WorthQueryApplicationProjection<Schema, SecondQuery>,
            Value: WorthQueryManagedDerivedValue,
            LinkKey: Eq,
        {
            self.inner.refresh_managed_derived_collection_pair_entry_from_batch_result::<MembershipQuery, FirstQuery, FirstResult, SecondQuery, SecondResult, Value, LinkKey>(view, product, key, first, batch, second_for, first_link, second_link, project)
        }
    }
}

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_application_query_observed_source_original_operation {
    use crate::domain_computation::primary_graph::WorthQueryOriginalOperationDenial as D;
    use worth_query_declaration::facade::application_query::ApplicationQueryBinding;
    use worth_query_declaration::facade::application_schema::ApplicationSchema;
    use worth_query_declaration::facade::application_schema::ApplicationStructuredValueBinding;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_query::observed_source::original_operation::WorthQueryOriginalSourceData;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_query::observed_source::original_operation::WorthQueryReadmittedOriginalSource;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    impl<S: ApplicationSchema + 'static> WorthQueryPrimaryGraphApplicationRuntime<S> {
        pub fn readmit_original_operation_source<B>(
            &self,
            phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
            __arg1: &crate::publication_boundary::WorthQueryProgramPublicationAccess,
            external: &worth_query_admission::facade::authenticated_principal::WorthQueryAuthenticatedExternalPrincipal<S>,
            request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
            data: &WorthQueryOriginalSourceData,
        ) -> Result<WorthQueryReadmittedOriginalSource<B::Query>, D>
        where
            B: ApplicationQueryBinding<S>,
            <B::ResultBinding as ApplicationStructuredValueBinding>::Value:
                crate::domain_computation::primary_graph::WorthQueryApplicationProjection<
                        S,
                        B::Query,
                    > + 'static,
        {
            self.inner
                .readmit_original_operation_source::<B>(phase, __arg1, external, request, data)
        }
    }
}

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_application_query_observed_source_prepared_expectation {
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_query::observed_source::prepared_expectation::pending::WorthQueryPendingSourceExpectation;
    use crate::domain_computation::primary_graph::WorthQueryAdmittedApplicationOperation;
    use worth_query_execution_lineage::facade::primary_graph::application_query::observed_source::WorthQueryObservedSource;
    use worth_query_execution_lineage::facade::primary_graph::application_query::observed_source::WorthQuerySourceExpectationDenial;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_query::observed_source::result_set::WorthQueryObservedResultSet;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
    where
        Schema: worth_query_installation::facade::ApplicationSchema,
    {
        pub fn bind_application_source_expectation<Binding, Scope>(&self, phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>, admission: &mut WorthQueryAdmittedApplicationOperation<
                Schema,
                Binding::Operation,
                Binding::Input,
                Scope,
            >, source: WorthQueryObservedSource<
                <Binding::SourceExpectation as worth_query_declaration::facade::application_operation::ApplicationMutationSourceExpectation<Schema>>::Query,
            >, input: &Binding::Input) -> Result<WorthQueryPendingSourceExpectation<<Binding::SourceExpectation as worth_query_declaration::facade::application_operation::ApplicationMutationSourceExpectation<Schema>>::Query>, WorthQuerySourceExpectationDenial>
        where
            Binding: worth_query_declaration::facade::application_operation::ApplicationMutationBinding<
                Schema,
        >,{
            self.inner
                .bind_application_source_expectation::<Binding, Scope>(
                    phase, admission, source, input,
                )
        }
        pub fn bind_application_result_set_expectation<Binding, Scope>(&self, phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>, admission: &mut WorthQueryAdmittedApplicationOperation<
                Schema,
                Binding::Operation,
                Binding::Input,
                Scope,
            >, result_set: WorthQueryObservedResultSet<
                <Binding::SourceExpectation as worth_query_declaration::facade::application_operation::ApplicationMutationSourceExpectation<Schema>>::Query,
            >, input: &Binding::Input) -> Result<WorthQueryPendingSourceExpectation<<Binding::SourceExpectation as worth_query_declaration::facade::application_operation::ApplicationMutationSourceExpectation<Schema>>::Query>, WorthQuerySourceExpectationDenial>
        where
            Binding: worth_query_declaration::facade::application_operation::ApplicationMutationBinding<
                Schema,
        >,{
            self.inner
                .bind_application_result_set_expectation::<Binding, Scope>(
                    phase, admission, result_set, input,
                )
        }
    }
}

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_application_query_observed_source_retained_dependencies {
    use crate::domain_computation::primary_graph::WorthQueryAdvancementPhase;
    use crate::domain_computation::primary_graph::WorthQueryApplicationOutputDemandSource;
    use worth_query_installation::facade::ApplicationSchema;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_query::observed_source::retained_dependencies::WorthQueryRetainedQueryDependencies;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_query::observed_source::retained_dependencies::WorthQueryRequiredSetCurrentnessDenial;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    impl<Schema: ApplicationSchema> WorthQuerySelectedProductOperation<'_, Schema> {
        pub fn retain_query_dependencies<Query, Value>(
            &self,
            phase: &WorthQueryAdvancementPhase<'_>,
            __arg1: &crate::publication_boundary::WorthQueryProgramPublicationAccess,
            source: &WorthQueryApplicationOutputDemandSource<Query, Value>,
        ) -> Result<WorthQueryRetainedQueryDependencies, WorthQueryRequiredSetCurrentnessDenial>
        {
            self.inner
                .retain_query_dependencies::<Query, Value>(phase, __arg1, source)
        }
        pub fn combine_query_dependencies<'a, I>(
            &self,
            phase: &WorthQueryAdvancementPhase<'_>,
            __arg1: &crate::publication_boundary::WorthQueryProgramPublicationAccess,
            parts: I,
        ) -> Result<WorthQueryRetainedQueryDependencies, WorthQueryRequiredSetCurrentnessDenial>
        where
            I: Iterator<Item = &'a WorthQueryRetainedQueryDependencies> + Clone,
        {
            self.inner
                .combine_query_dependencies::<I>(phase, __arg1, parts)
        }
    }
}

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_application_query_observed_source_retained_dependencies_comparison {
    use crate::domain_computation::primary_graph::WorthQueryOutputDemandSettlement;
    use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
    use crate::domain_computation::primary_graph::WorthQueryAdvancementPhase;
    use worth_query_installation::facade::ApplicationSchema;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_query::observed_source::retained_dependencies::WorthQueryRetainedQueryDependencies;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_query::observed_source::retained_dependencies::WorthQueryRequiredSetCurrentnessDenial;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    impl<Schema: ApplicationSchema> WorthQuerySelectedProductOperation<'_, Schema> {
        pub fn require_current_required_set<'s>(
            &self,
            phase: &WorthQueryAdvancementPhase<'_>,
            __arg1: &crate::publication_boundary::WorthQueryProgramPublicationAccess,
            request: &WorthQueryRequestScope,
            settlements: impl IntoIterator<Item = &'s WorthQueryOutputDemandSettlement>,
            dependencies: &WorthQueryRetainedQueryDependencies,
        ) -> Result<(), WorthQueryRequiredSetCurrentnessDenial> {
            self.inner.require_current_required_set(
                phase,
                __arg1,
                request,
                settlements,
                dependencies,
            )
        }
    }
}
