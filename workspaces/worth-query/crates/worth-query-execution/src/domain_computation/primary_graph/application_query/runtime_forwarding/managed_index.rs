//! Thin forwarding shims for `application_query` methods onto `worth-query-execution-query`.

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_application_query_managed_index_receiver_eviction {
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_query::managed_index::handle::WorthQueryManagedIndex;
    use worth_query_declaration::facade::application_query::ApplicationManagedIndexBinding;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_query::managed_index::denial::WorthQueryManagedIndexDenial as Denial;
    use crate::domain_computation::primary_graph::application_query::WorthQueryAdmittedApplicationQueryPlan;
    use crate::domain_computation::primary_graph::WorthQueryAdvancementPhase;
    use worth_query_declaration::facade::application_query::ApplicationQueryBinding;
    use worth_query_declaration::facade::application_query::ApplicationQueryScopeBinding;
    use worth_query_declaration::facade::application_schema::ApplicationSchema;
    use worth_query_declaration::facade::application_schema::ApplicationStructuredValueBinding;
    use worth_query_installation::facade::WorthQueryInstalledApplicationQueryBinding;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    type BoundPlan<'a, S, B> = WorthQueryAdmittedApplicationQueryPlan<
    'a, S, <B as ApplicationQueryBinding<S>>::Query,
    <<B as ApplicationQueryBinding<S>>::ParameterBinding as ApplicationStructuredValueBinding>::Value,
    ResultValue<S, B>, <B as ApplicationQueryBinding<S>>::Principal,
    <B as ApplicationQueryBinding<S>>::PrincipalIdentity,
    <<B as ApplicationQueryBinding<S>>::ScopeBinding as ApplicationQueryScopeBinding<S>>::Scope,
>;
    type ResultValue<S, B> =
    <<B as ApplicationQueryBinding<S>>::ResultBinding as ApplicationStructuredValueBinding>::Value;
    impl<S: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<S> {
        pub fn evict_application_managed_index<
            B: ApplicationQueryBinding<S>,
            I: ApplicationManagedIndexBinding<S>,
        >(
            &self,
            phase: &WorthQueryAdvancementPhase<'_>,
            binding: &WorthQueryInstalledApplicationQueryBinding<S, B>,
            plan: BoundPlan<'_, S, B>,
            index: &WorthQueryManagedIndex<S, I>,
        ) -> Result<(), Denial> {
            self.inner
                .evict_application_managed_index::<B, I>(phase, binding, plan, index)
        }
    }
}

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_application_query_managed_index_receiver_lifecycle {
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_query::managed_index::handle::WorthQueryManagedIndex;
    use worth_query_declaration::facade::application_query::ApplicationManagedIndexBinding;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_query::managed_index::denial::WorthQueryManagedIndexDenial as Denial;
    use crate::domain_computation::primary_graph::application_query::WorthQueryAdmittedApplicationQueryPlan;
    use crate::domain_computation::primary_graph::WorthQueryAdvancementPhase;
    use worth_query_admission::facade::authenticated_principal::WorthQueryAuthenticatedExternalPrincipal;
    use worth_query_declaration::facade::application_query::ApplicationQueryBinding;
    use worth_query_declaration::facade::application_query::ApplicationQueryScopeBinding;
    use worth_query_declaration::facade::application_schema::ApplicationSchema;
    use worth_query_declaration::facade::application_schema::ApplicationStructuredValueBinding;
    use worth_query_installation::facade::WorthQueryInstalledApplicationQueryBinding;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    type BoundPlan<'a, S, B> = WorthQueryAdmittedApplicationQueryPlan<
    'a, S, <B as ApplicationQueryBinding<S>>::Query,
    <<B as ApplicationQueryBinding<S>>::ParameterBinding as ApplicationStructuredValueBinding>::Value,
    ResultValue<S, B>, <B as ApplicationQueryBinding<S>>::Principal,
    <B as ApplicationQueryBinding<S>>::PrincipalIdentity,
    <<B as ApplicationQueryBinding<S>>::ScopeBinding as ApplicationQueryScopeBinding<S>>::Scope,
>;
    type ResultValue<S, B> =
    <<B as ApplicationQueryBinding<S>>::ResultBinding as ApplicationStructuredValueBinding>::Value;
    impl<S: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<S> {
        pub fn open_application_managed_index<
            B: ApplicationQueryBinding<S>,
            I: ApplicationManagedIndexBinding<S>,
        >(
            &self,
            phase: &WorthQueryAdvancementPhase<'_>,
            binding: &WorthQueryInstalledApplicationQueryBinding<S, B>,
            plan: BoundPlan<'_, S, B>,
        ) -> Result<WorthQueryManagedIndex<S, I>, Denial> {
            self.inner
                .open_application_managed_index::<B, I>(phase, binding, plan)
        }
        pub fn ensure_application_managed_index_ready<
            B: ApplicationQueryBinding<S>,
            I: ApplicationManagedIndexBinding<S>,
        >(
            &self,
            phase: &WorthQueryAdvancementPhase<'_>,
            binding: &WorthQueryInstalledApplicationQueryBinding<S, B>,
            external: &WorthQueryAuthenticatedExternalPrincipal<S>,
            plan: BoundPlan<'_, S, B>,
        ) -> Result<WorthQueryManagedIndex<S, I>, Denial> {
            self.inner
                .ensure_application_managed_index_ready::<B, I>(phase, binding, external, plan)
        }
        pub fn rebuild_application_managed_index<
            B: ApplicationQueryBinding<S>,
            I: ApplicationManagedIndexBinding<S>,
        >(
            &self,
            phase: &WorthQueryAdvancementPhase<'_>,
            binding: &WorthQueryInstalledApplicationQueryBinding<S, B>,
            external: &WorthQueryAuthenticatedExternalPrincipal<S>,
            plan: BoundPlan<'_, S, B>,
            index: &WorthQueryManagedIndex<S, I>,
        ) -> Result<(), Denial> {
            self.inner
                .rebuild_application_managed_index::<B, I>(phase, binding, external, plan, index)
        }
    }
}

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_application_query_managed_index_receiver_watch_next {
    use worth_query_declaration::facade::application_query::ApplicationManagedIndexBinding;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_query::managed_index::denial::WorthQueryManagedIndexDenial as Denial;
    use crate::domain_computation::primary_graph::application_query::WorthQueryAdmittedApplicationQueryPlan;
    use crate::domain_computation::primary_graph::application_query::WorthQueryApplicationProjection;
    use crate::domain_computation::primary_graph::WorthQueryAdvancementPhase;
    use worth_query_admission::facade::authenticated_principal::WorthQueryAuthenticatedExternalPrincipal;
    use worth_query_declaration::facade::application_query::ApplicationQueryBinding;
    use worth_query_declaration::facade::application_query::ApplicationQueryScopeBinding;
    use worth_query_declaration::facade::application_schema::ApplicationSchema;
    use worth_query_declaration::facade::application_schema::ApplicationStructuredValueBinding;
    use worth_query_installation::facade::WorthQueryInstalledApplicationQueryBinding;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_query::managed_index::receiver::watch::WorthQueryManagedIndexWatch;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_query::managed_index::receiver::watch::WorthQueryManagedIndexWatchOutcome;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    type ResultValue<S, B> =
    <<B as ApplicationQueryBinding<S>>::ResultBinding as ApplicationStructuredValueBinding>::Value;
    type BoundPlan<'a, S, B> = WorthQueryAdmittedApplicationQueryPlan<
    'a, S, <B as ApplicationQueryBinding<S>>::Query,
    <<B as ApplicationQueryBinding<S>>::ParameterBinding as ApplicationStructuredValueBinding>::Value,
    ResultValue<S, B>, <B as ApplicationQueryBinding<S>>::Principal,
    <B as ApplicationQueryBinding<S>>::PrincipalIdentity,
    <<B as ApplicationQueryBinding<S>>::ScopeBinding as ApplicationQueryScopeBinding<S>>::Scope,
>;
    impl<S: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<S> {
        pub fn next_application_managed_index_watch<B, I>(
            &self,
            phase: &WorthQueryAdvancementPhase<'_>,
            binding: &WorthQueryInstalledApplicationQueryBinding<S, B>,
            external: &WorthQueryAuthenticatedExternalPrincipal<S>,
            plan: BoundPlan<'_, S, B>,
            watch: &mut WorthQueryManagedIndexWatch<S, I>,
        ) -> Result<WorthQueryManagedIndexWatchOutcome<B::Query, ResultValue<S, B>>, Denial>
        where
            B: ApplicationQueryBinding<S>,
            I: ApplicationManagedIndexBinding<S>,
            ResultValue<S, B>: WorthQueryApplicationProjection<S, B::Query>,
        {
            self.inner
                .next_application_managed_index_watch::<B, I>(phase, binding, external, plan, watch)
        }
    }
}

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_application_query_managed_index_receiver_watch_open {
    use worth_query_declaration::facade::application_query::ApplicationManagedIndexBinding;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_query::managed_index::denial::WorthQueryManagedIndexDenial as Denial;
    use crate::domain_computation::primary_graph::application_query::WorthQueryAdmittedApplicationQueryPlan;
    use crate::domain_computation::primary_graph::application_query::WorthQueryApplicationOneShotResult;
    use crate::domain_computation::primary_graph::application_query::WorthQueryApplicationProjection;
    use crate::domain_computation::primary_graph::WorthQueryAdvancementPhase;
    use worth_query_admission::facade::authenticated_principal::WorthQueryAuthenticatedExternalPrincipal;
    use worth_query_declaration::facade::application_query::ApplicationQueryBinding;
    use worth_query_declaration::facade::application_query::ApplicationQueryScopeBinding;
    use worth_query_declaration::facade::application_schema::ApplicationSchema;
    use worth_query_declaration::facade::application_schema::ApplicationStructuredValueBinding;
    use worth_query_installation::facade::WorthQueryInstalledApplicationQueryBinding;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_query::managed_index::receiver::watch::WorthQueryManagedIndexWatch;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    type ResultValue<S, B> =
    <<B as ApplicationQueryBinding<S>>::ResultBinding as ApplicationStructuredValueBinding>::Value;
    type BoundPlan<'a, S, B> = WorthQueryAdmittedApplicationQueryPlan<
    'a, S, <B as ApplicationQueryBinding<S>>::Query,
    <<B as ApplicationQueryBinding<S>>::ParameterBinding as ApplicationStructuredValueBinding>::Value,
    ResultValue<S, B>, <B as ApplicationQueryBinding<S>>::Principal,
    <B as ApplicationQueryBinding<S>>::PrincipalIdentity,
    <<B as ApplicationQueryBinding<S>>::ScopeBinding as ApplicationQueryScopeBinding<S>>::Scope,
>;
    impl<S: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<S> {
        pub fn open_application_managed_index_watch<B, I>(
            &self,
            phase: &WorthQueryAdvancementPhase<'_>,
            binding: &WorthQueryInstalledApplicationQueryBinding<S, B>,
            external: &WorthQueryAuthenticatedExternalPrincipal<S>,
            plan: BoundPlan<'_, S, B>,
        ) -> Result<
            (
                WorthQueryApplicationOneShotResult<B::Query, ResultValue<S, B>>,
                WorthQueryManagedIndexWatch<S, I>,
            ),
            Denial,
        >
        where
            B: ApplicationQueryBinding<S>,
            I: ApplicationManagedIndexBinding<S>,
            ResultValue<S, B>: WorthQueryApplicationProjection<S, B::Query>,
        {
            self.inner
                .open_application_managed_index_watch::<B, I>(phase, binding, external, plan)
        }
    }
}

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_application_query_derived_view_registration {
    use worth_query_declaration::facade::application_query::ApplicationDerivedViewDefinition;
    use worth_query_declaration::facade::application_schema::ApplicationSchema;
    use worth_query_installation::facade::WorthQueryInstalledApplicationQuery;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_query::derived_view::retention::WorthQueryManagedDerivedViewDenial as Denial;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_query::derived_view::WorthQueryManagedDerivedValue;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_query::derived_view::WorthQueryManagedDerivedView;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_query::derived_view::WorthQueryManagedDerivedViewSnapshot;
    use crate::basis::WorthQueryProductBranchLease;
    use crate::domain_computation::primary_graph::application_query::WorthQueryApplicationOneShotResult;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
    where
        Schema: ApplicationSchema,
    {
        pub fn open_managed_derived_view<Query, Parameters, QueryResult, Scope, Value>(
            &self,
            definition: &ApplicationDerivedViewDefinition<
                Schema,
                Query,
                Parameters,
                QueryResult,
                Scope,
            >,
            query: &WorthQueryInstalledApplicationQuery<
                Schema,
                Query,
                Parameters,
                QueryResult,
                Scope,
            >,
            product: &WorthQueryProductBranchLease,
        ) -> Result<WorthQueryManagedDerivedView<Query, Value>, Denial>
        where
            Value: WorthQueryManagedDerivedValue,
        {
            self.inner
                .open_managed_derived_view::<Query, Parameters, QueryResult, Scope, Value>(
                    definition, query, product,
                )
        }
        pub fn observe_managed_derived_view<Query, Value>(
            &self,
            view: &WorthQueryManagedDerivedView<Query, Value>,
        ) -> Result<WorthQueryManagedDerivedViewSnapshot<Value>, Denial>
        where
            Value: WorthQueryManagedDerivedValue,
        {
            self.inner
                .observe_managed_derived_view::<Query, Value>(view)
        }
        pub fn reconstruct_managed_derived_view<Query, QueryResult, Value>(
            &self,
            view: &WorthQueryManagedDerivedView<Query, Value>,
            product: &WorthQueryProductBranchLease,
            result: &WorthQueryApplicationOneShotResult<Query, QueryResult>,
            project: impl FnMut(&QueryResult) -> Value,
        ) -> Result<(), Denial>
        where
            Value: WorthQueryManagedDerivedValue,
        {
            self.inner
                .reconstruct_managed_derived_view::<Query, QueryResult, Value>(
                    view, product, result, project,
                )
        }
    }
}

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_application_query_derived_view_registration_collection {
    use worth_query_declaration::facade::application_query::ApplicationDerivedViewDefinition;
    use worth_query_declaration::facade::application_schema::ApplicationSchema;
    use worth_query_installation::facade::WorthQueryInstalledApplicationQuery;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_query::derived_view::registration::Denial;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_query::derived_view::registration::WorthQueryManagedDerivedValue;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_query::derived_view::registration::WorthQueryManagedDerivedView;
    use crate::basis::WorthQueryProductBranchLease;
    use crate::domain_computation::primary_graph::application_query::WorthQueryAdmittedApplicationQueryPlan;
    use crate::domain_computation::primary_graph::application_query::WorthQueryApplicationProjection;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
    where
        Schema: ApplicationSchema,
    {
        pub fn open_managed_derived_collection<
            MembershipQuery,
            MembershipParameters,
            MembershipResult,
            MembershipScope,
            EntryQuery,
            EntryParameters,
            EntryResult,
            EntryScope,
            Value,
        >(
            &self,
            definition: &ApplicationDerivedViewDefinition<
                Schema,
                MembershipQuery,
                MembershipParameters,
                MembershipResult,
                MembershipScope,
            >,
            membership_query: &WorthQueryInstalledApplicationQuery<
                Schema,
                MembershipQuery,
                MembershipParameters,
                MembershipResult,
                MembershipScope,
            >,
            entry_query: &WorthQueryInstalledApplicationQuery<
                Schema,
                EntryQuery,
                EntryParameters,
                EntryResult,
                EntryScope,
            >,
            product: &WorthQueryProductBranchLease,
        ) -> Result<WorthQueryManagedDerivedView<MembershipQuery, Value>, Denial>
        where
            Value: WorthQueryManagedDerivedValue,
        {
            self.inner.open_managed_derived_collection::<MembershipQuery, MembershipParameters, MembershipResult, MembershipScope, EntryQuery, EntryParameters, EntryResult, EntryScope, Value>(definition, membership_query, entry_query, product)
        }
        pub fn refresh_managed_derived_collection_entry<
            'a,
            MembershipQuery,
            EntryQuery,
            EntryParameters,
            EntryResult,
            Principal,
            PrincipalIdentity,
            EntryScope,
            Value,
        >(
            &self,
            execution: &worth_execution::ExecutionRequest<'_, '_>,
            view: &WorthQueryManagedDerivedView<MembershipQuery, Value>,
            product: &WorthQueryProductBranchLease,
            key: &worth_query_execution_query::facade::domain_computation::primary_graph::application_query::derived_view::WorthQueryManagedDerivedViewKey,
            plan: WorthQueryAdmittedApplicationQueryPlan<
                'a,
                Schema,
                EntryQuery,
                EntryParameters,
                EntryResult,
                Principal,
                PrincipalIdentity,
                EntryScope,
            >,
            project: impl FnOnce(&EntryResult) -> Value,
        ) -> Result<std::sync::Arc<Value>, Denial>
        where
            EntryResult: WorthQueryApplicationProjection<Schema, EntryQuery>,
            Value: WorthQueryManagedDerivedValue,
        {
            self.inner.refresh_managed_derived_collection_entry::<MembershipQuery, EntryQuery, EntryParameters, EntryResult, Principal, PrincipalIdentity, EntryScope, Value>(execution, view, product, key, plan, project)
        }
    }
}
