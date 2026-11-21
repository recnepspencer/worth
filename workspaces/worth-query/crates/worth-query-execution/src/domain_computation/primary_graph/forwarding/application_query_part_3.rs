//! Thin forwarding shims for `application_query` methods onto `worth-query-execution-query`.

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_application_query_derived_view_registration_collection_pair {
    use worth_query_declaration::facade::application_query::ApplicationDerivedViewDefinition;
    use worth_query_declaration::facade::application_schema::ApplicationSchema;
    use worth_query_installation::facade::WorthQueryInstalledApplicationQuery;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_query::derived_view::registration::collection::Denial;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_query::derived_view::registration::collection::WorthQueryAdmittedApplicationQueryPlan;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_query::derived_view::registration::collection::WorthQueryApplicationProjection;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_query::derived_view::registration::collection::WorthQueryManagedDerivedValue;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_query::derived_view::registration::collection::WorthQueryManagedDerivedView;
    use crate::basis::WorthQueryProductBranchLease;
    use crate::domain_computation::primary_graph::application_query::derived_view::WorthQueryManagedDerivedViewKey;
    use crate::domain_computation::primary_graph::application_query::WorthQueryApplicationOneShotResult;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: ApplicationSchema, {
        pub fn open_managed_derived_collection_pair<
            MembershipQuery,
            MembershipParameters,
            MembershipResult,
            MembershipScope,
            FirstQuery,
            FirstParameters,
            FirstResult,
            FirstScope,
            SecondQuery,
            SecondParameters,
            SecondResult,
            SecondScope,
            Value,
        >(&self, definition: &ApplicationDerivedViewDefinition<
                Schema,
                MembershipQuery,
                MembershipParameters,
                MembershipResult,
                MembershipScope,
            >, membership_query: &WorthQueryInstalledApplicationQuery<
                Schema,
                MembershipQuery,
                MembershipParameters,
                MembershipResult,
                MembershipScope,
            >, first_query: &WorthQueryInstalledApplicationQuery<
                Schema,
                FirstQuery,
                FirstParameters,
                FirstResult,
                FirstScope,
            >, second_query: &WorthQueryInstalledApplicationQuery<
                Schema,
                SecondQuery,
                SecondParameters,
                SecondResult,
                SecondScope,
            >, product: &WorthQueryProductBranchLease) -> Result<WorthQueryManagedDerivedView<MembershipQuery, Value>, Denial>
        where
            Value: WorthQueryManagedDerivedValue, {
            self.inner.open_managed_derived_collection_pair::<MembershipQuery, MembershipParameters, MembershipResult, MembershipScope, FirstQuery, FirstParameters, FirstResult, FirstScope, SecondQuery, SecondParameters, SecondResult, SecondScope, Value>(definition, membership_query, first_query, second_query, product)
        }
        pub fn open_owner_sized_managed_derived_collection_pair<
            MembershipQuery,
            MembershipParameters,
            MembershipResult,
            MembershipScope,
            FirstQuery,
            FirstParameters,
            FirstResult,
            FirstScope,
            SecondQuery,
            SecondParameters,
            SecondResult,
            SecondScope,
            Value,
        >(&self, name: &'static str, membership_query: &WorthQueryInstalledApplicationQuery<
                Schema,
                MembershipQuery,
                MembershipParameters,
                MembershipResult,
                MembershipScope,
            >, first_query: &WorthQueryInstalledApplicationQuery<
                Schema,
                FirstQuery,
                FirstParameters,
                FirstResult,
                FirstScope,
            >, second_query: &WorthQueryInstalledApplicationQuery<
                Schema,
                SecondQuery,
                SecondParameters,
                SecondResult,
                SecondScope,
            >, product: &WorthQueryProductBranchLease) -> Result<WorthQueryManagedDerivedView<MembershipQuery, Value>, Denial>
        where
            Value: WorthQueryManagedDerivedValue, {
            self.inner.open_owner_sized_managed_derived_collection_pair::<MembershipQuery, MembershipParameters, MembershipResult, MembershipScope, FirstQuery, FirstParameters, FirstResult, FirstScope, SecondQuery, SecondParameters, SecondResult, SecondScope, Value>(name, membership_query, first_query, second_query, product)
        }
        #[allow(clippy::too_many_arguments)]
        pub fn refresh_managed_derived_collection_pair_entry<
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
        >(&self, execution: &worth_execution::ExecutionRequest<'_, '_>, view: &WorthQueryManagedDerivedView<MembershipQuery, Value>, product: &WorthQueryProductBranchLease, key: &WorthQueryManagedDerivedViewKey, first: WorthQueryAdmittedApplicationQueryPlan<
                'a,
                Schema,
                FirstQuery,
                FirstParameters,
                FirstResult,
                FirstPrincipal,
                FirstPrincipalIdentity,
                FirstScope,
            >, second_for: impl FnOnce(
                &FirstResult,
            ) -> Result<
                WorthQueryApplicationOneShotResult<SecondQuery, SecondResult>,
                Denial,
            >, first_link: impl FnOnce(&FirstResult) -> LinkKey, second_link: impl FnOnce(&SecondResult) -> LinkKey, project: impl FnOnce(&FirstResult, &SecondResult) -> Value) -> Result<std::sync::Arc<Value>, Denial>
        where
            FirstQuery: 'a,
            FirstParameters: 'a,
            FirstResult: WorthQueryApplicationProjection<Schema, FirstQuery> + 'a,
            FirstPrincipal: 'a,
            FirstPrincipalIdentity: 'a,
            FirstScope: 'a,
            SecondResult: WorthQueryApplicationProjection<Schema, SecondQuery> + 'a,
            Value: WorthQueryManagedDerivedValue,
            LinkKey: Eq, {
            self.inner.refresh_managed_derived_collection_pair_entry::<MembershipQuery, FirstQuery, FirstParameters, FirstResult, FirstPrincipal, FirstPrincipalIdentity, FirstScope, SecondQuery, SecondResult, Value, LinkKey>(execution, view, product, key, first, second_for, first_link, second_link, project)
        }
    }
}

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_application_query_derived_view_registration_collection_pair_reconcile {
    use worth_query_declaration::facade::application_schema::ApplicationSchema;
    use worth_relational::facade::identity::EntityId;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_query::derived_view::registration::collection::pair::Denial;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_query::derived_view::registration::collection::pair::WorthQueryApplicationProjection;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_query::derived_view::registration::collection::pair::WorthQueryManagedDerivedValue;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_query::derived_view::registration::collection::pair::WorthQueryManagedDerivedView;
    use crate::basis::WorthQueryProductBranchLease;
    use crate::domain_computation::primary_graph::application_query::derived_view::retention::WorthQueryManagedDerivedMemberToken;
    use crate::domain_computation::primary_graph::application_query::derived_view::WorthQueryManagedDerivedViewReconciliation;
    use crate::domain_computation::primary_graph::application_query::WorthQueryApplicationOneShotResult;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: ApplicationSchema, {
        #[allow(clippy::too_many_arguments)]
        pub fn reconcile_managed_derived_collection_pair_lazy<
            MembershipQuery,
            MembershipResult,
            Member,
            FirstQuery,
            FirstResult,
            SecondQuery,
            SecondResult,
            Value,
            LinkKey,
        >(&self, view: &WorthQueryManagedDerivedView<MembershipQuery, Value>, product: &WorthQueryProductBranchLease, membership_result: &WorthQueryApplicationOneShotResult<MembershipQuery, MembershipResult>, members: impl Fn(&MembershipResult) -> Vec<(EntityId, Member)>, first_for: impl FnMut(
                &Member,
            ) -> Result<
                WorthQueryApplicationOneShotResult<FirstQuery, FirstResult>,
                Denial,
            >, second_for: impl FnMut(
                &FirstResult,
            ) -> Result<
                WorthQueryApplicationOneShotResult<SecondQuery, SecondResult>,
                Denial,
            >, first_link: impl Fn(&FirstResult) -> LinkKey, second_link: impl Fn(&SecondResult) -> LinkKey, project: impl FnMut(&FirstResult, &SecondResult) -> Value) -> Result<WorthQueryManagedDerivedViewReconciliation, Denial>
        where
            FirstResult: WorthQueryApplicationProjection<Schema, FirstQuery>,
            SecondResult: WorthQueryApplicationProjection<Schema, SecondQuery>,
            Value: WorthQueryManagedDerivedValue,
            Member: WorthQueryManagedDerivedMemberToken,
            LinkKey: Eq, {
            self.inner.reconcile_managed_derived_collection_pair_lazy::<MembershipQuery, MembershipResult, Member, FirstQuery, FirstResult, SecondQuery, SecondResult, Value, LinkKey>(view, product, membership_result, members, first_for, second_for, first_link, second_link, project)
        }
    }
}

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_application_query_derived_view_registration_collection_pair_reconstruct_dependent {
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_query::derived_view::registration::collection::pair::Denial;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_query::derived_view::registration::collection::pair::WorthQueryApplicationProjection;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_query::derived_view::registration::collection::pair::WorthQueryManagedDerivedValue;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_query::derived_view::registration::collection::pair::WorthQueryManagedDerivedView;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_query::derived_view::registration::collection::pair::WorthQueryManagedDerivedViewKey;
    use crate::basis::WorthQueryProductBranchLease;
    use crate::domain_computation::primary_graph::application_query::derived_view::retention::WorthQueryManagedDerivedMemberToken;
    use crate::domain_computation::primary_graph::application_query::WorthQueryApplicationOneShotResult;
    use crate::domain_computation::primary_graph::application_query::WorthQueryApplicationQueryBatchAdmission;
    use crate::domain_computation::primary_graph::application_query::WorthQueryDependentPairReadContext;
    use worth_execution::ExecutionRequest;
    use worth_query_declaration::facade::application_schema::ApplicationSchema;
    use worth_relational::facade::identity::EntityId;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    impl<S: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<S> {
        #[allow(clippy::too_many_arguments)]
        pub fn reconstruct_managed_derived_collection_pair_dependent_in_batch<
            MQ,
            MR,
            Member,
            FQ,
            FR,
            SQ,
            SR,
            Value,
            Link,
        >(&self, request: ExecutionRequest<'_, '_>, view: &WorthQueryManagedDerivedView<MQ, Value>, product: &WorthQueryProductBranchLease, membership_result: &WorthQueryApplicationOneShotResult<MQ, MR>, members: impl Fn(&MR) -> Vec<(EntityId, Member)>, batch: &WorthQueryApplicationQueryBatchAdmission, first_for: impl FnMut(
                &Member,
                &mut WorthQueryDependentPairReadContext<'_, '_, '_, '_, '_, S, FQ, FR>,
            ) -> Result<(), Denial>, second_for: impl FnMut(
                &FR,
                &mut WorthQueryDependentPairReadContext<'_, '_, '_, '_, '_, S, SQ, SR>,
            ) -> Result<(), Denial>, first_link: impl Fn(&FR) -> Link, second_link: impl Fn(&SR) -> Link, project: impl FnMut(&FR, &SR) -> Value) -> Result<Vec<WorthQueryManagedDerivedViewKey>, Denial>
        where
            FR: WorthQueryApplicationProjection<S, FQ>,
            SR: WorthQueryApplicationProjection<S, SQ>,
            Member: WorthQueryManagedDerivedMemberToken,
            Value: WorthQueryManagedDerivedValue,
            Link: Eq, {
            self.inner.reconstruct_managed_derived_collection_pair_dependent_in_batch::<MQ, MR, Member, FQ, FR, SQ, SR, Value, Link>(request, view, product, membership_result, members, batch, first_for, second_for, first_link, second_link, project)
        }
    }
}

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_application_query_derived_view_registration_collection_pair_reconstruct_lazy {
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_query::derived_view::registration::collection::pair::Denial;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_query::derived_view::registration::collection::pair::WorthQueryApplicationOneShotResult;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_query::derived_view::registration::collection::pair::WorthQueryApplicationProjection;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_query::derived_view::registration::collection::pair::WorthQueryManagedDerivedValue;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_query::derived_view::registration::collection::pair::WorthQueryManagedDerivedView;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_query::derived_view::registration::collection::pair::WorthQueryManagedDerivedViewKey;
    use crate::basis::WorthQueryProductBranchLease;
    use crate::domain_computation::primary_graph::application_query::derived_view::retention::WorthQueryManagedDerivedMemberToken;
    use crate::domain_computation::primary_graph::WorthQueryAdvancementPhase;
    use worth_query_declaration::facade::application_schema::ApplicationSchema;
    use worth_relational::facade::identity::EntityId;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
        #[allow(clippy::too_many_arguments)]
        pub fn reconstruct_managed_derived_collection_pair_lazy<
            MQ,
            MR,
            Member,
            FQ,
            FR,
            SQ,
            SR,
            Value,
            Link,
        >(&self, phase: &WorthQueryAdvancementPhase<'_>, view: &WorthQueryManagedDerivedView<MQ, Value>, product: &WorthQueryProductBranchLease, membership_result: &WorthQueryApplicationOneShotResult<MQ, MR>, members: impl Fn(&MR) -> Vec<(EntityId, Member)>, first_for: impl FnMut(&Member) -> Result<WorthQueryApplicationOneShotResult<FQ, FR>, Denial>, second_for: impl FnMut(&FR) -> Result<WorthQueryApplicationOneShotResult<SQ, SR>, Denial>, first_link: impl Fn(&FR) -> Link, second_link: impl Fn(&SR) -> Link, project: impl FnMut(&FR, &SR) -> Value) -> Result<Vec<WorthQueryManagedDerivedViewKey>, Denial>
        where
            FR: WorthQueryApplicationProjection<Schema, FQ>,
            SR: WorthQueryApplicationProjection<Schema, SQ>,
            Member: WorthQueryManagedDerivedMemberToken,
            Value: WorthQueryManagedDerivedValue,
            Link: Eq, {
            self.inner.reconstruct_managed_derived_collection_pair_lazy::<MQ, MR, Member, FQ, FR, SQ, SR, Value, Link>(phase, view, product, membership_result, members, first_for, second_for, first_link, second_link, project)
        }
    }
}

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_application_query_derived_view_registration_collection_pair_reconstruct_parallel {
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_query::derived_view::registration::collection::pair::Denial;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_query::derived_view::registration::collection::pair::WorthQueryApplicationProjection;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_query::derived_view::registration::collection::pair::WorthQueryManagedDerivedValue;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_query::derived_view::registration::collection::pair::WorthQueryManagedDerivedView;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_query::derived_view::registration::collection::pair::WorthQueryManagedDerivedViewKey;
    use crate::basis::WorthQueryProductBranchLease;
    use crate::domain_computation::primary_graph::application_query::derived_view::retention::WorthQueryManagedDerivedMemberToken;
    use crate::domain_computation::primary_graph::application_query::WorthQueryApplicationOneShotResult;
    use crate::domain_computation::primary_graph::application_query::WorthQueryDerivedPairReadPlans;
    use worth_execution::ExecutionRequest;
    use worth_query_declaration::facade::application_schema::ApplicationSchema;
    use worth_relational::facade::identity::EntityId;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
        #[allow(clippy::too_many_arguments)]
        pub fn reconstruct_managed_derived_collection_pair<
            'a,
            MQ,
            MR,
            Member,
            FQ: 'a,
            FP: 'a,
            FR,
            SQ: 'a,
            SP: 'a,
            SR,
            A: 'a,
            I: 'a,
            C: 'a,
            Value,
            Link,
        >(&self, request: ExecutionRequest<'_, '_>, view: &WorthQueryManagedDerivedView<MQ, Value>, product: &WorthQueryProductBranchLease, membership_result: &WorthQueryApplicationOneShotResult<MQ, MR>, members: impl Fn(&MR) -> Vec<(EntityId, Member)>, prepare: impl FnMut(
                &Member,
            ) -> Result<
                WorthQueryDerivedPairReadPlans<'a, Schema, FQ, FP, FR, SQ, SP, SR, A, I, C>,
                Denial,
            >, first_link: impl Fn(&FR) -> Link, second_link: impl Fn(&SR) -> Link, project: impl FnMut(&FR, &SR) -> Value) -> Result<Vec<WorthQueryManagedDerivedViewKey>, Denial>
        where
            FR: WorthQueryApplicationProjection<Schema, FQ> + 'a,
            SR: WorthQueryApplicationProjection<Schema, SQ> + 'a,
            Member: WorthQueryManagedDerivedMemberToken,
            Value: WorthQueryManagedDerivedValue,
            Link: Eq, {
            self.inner.reconstruct_managed_derived_collection_pair::<MQ, MR, Member, FQ, FP, FR, SQ, SP, SR, A, I, C, Value, Link>(request, view, product, membership_result, members, prepare, first_link, second_link, project)
        }
    }
}
