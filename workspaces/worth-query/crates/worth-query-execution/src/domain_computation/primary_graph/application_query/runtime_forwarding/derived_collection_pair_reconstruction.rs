//! Thin forwarding shims for `application_query` methods onto `worth-query-execution-query`.

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
        >(
            &self,
            request: ExecutionRequest<'_, '_>,
            view: &WorthQueryManagedDerivedView<MQ, Value>,
            product: &WorthQueryProductBranchLease,
            membership_result: &WorthQueryApplicationOneShotResult<MQ, MR>,
            members: impl Fn(&MR) -> Vec<(EntityId, Member)>,
            batch: &WorthQueryApplicationQueryBatchAdmission,
            first_for: impl FnMut(
                &Member,
                &mut WorthQueryDependentPairReadContext<'_, '_, '_, '_, '_, S, FQ, FR>,
            ) -> Result<(), Denial>,
            second_for: impl FnMut(
                &FR,
                &mut WorthQueryDependentPairReadContext<'_, '_, '_, '_, '_, S, SQ, SR>,
            ) -> Result<(), Denial>,
            first_link: impl Fn(&FR) -> Link,
            second_link: impl Fn(&SR) -> Link,
            project: impl FnMut(&FR, &SR) -> Value,
        ) -> Result<Vec<WorthQueryManagedDerivedViewKey>, Denial>
        where
            FR: WorthQueryApplicationProjection<S, FQ>,
            SR: WorthQueryApplicationProjection<S, SQ>,
            Member: WorthQueryManagedDerivedMemberToken,
            Value: WorthQueryManagedDerivedValue,
            Link: Eq,
        {
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
        >(
            &self,
            phase: &WorthQueryAdvancementPhase<'_>,
            view: &WorthQueryManagedDerivedView<MQ, Value>,
            product: &WorthQueryProductBranchLease,
            membership_result: &WorthQueryApplicationOneShotResult<MQ, MR>,
            members: impl Fn(&MR) -> Vec<(EntityId, Member)>,
            first_for: impl FnMut(&Member) -> Result<WorthQueryApplicationOneShotResult<FQ, FR>, Denial>,
            second_for: impl FnMut(&FR) -> Result<WorthQueryApplicationOneShotResult<SQ, SR>, Denial>,
            first_link: impl Fn(&FR) -> Link,
            second_link: impl Fn(&SR) -> Link,
            project: impl FnMut(&FR, &SR) -> Value,
        ) -> Result<Vec<WorthQueryManagedDerivedViewKey>, Denial>
        where
            FR: WorthQueryApplicationProjection<Schema, FQ>,
            SR: WorthQueryApplicationProjection<Schema, SQ>,
            Member: WorthQueryManagedDerivedMemberToken,
            Value: WorthQueryManagedDerivedValue,
            Link: Eq,
        {
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
        >(
            &self,
            request: ExecutionRequest<'_, '_>,
            view: &WorthQueryManagedDerivedView<MQ, Value>,
            product: &WorthQueryProductBranchLease,
            membership_result: &WorthQueryApplicationOneShotResult<MQ, MR>,
            members: impl Fn(&MR) -> Vec<(EntityId, Member)>,
            prepare: impl FnMut(
                &Member,
            ) -> Result<
                WorthQueryDerivedPairReadPlans<'a, Schema, FQ, FP, FR, SQ, SP, SR, A, I, C>,
                Denial,
            >,
            first_link: impl Fn(&FR) -> Link,
            second_link: impl Fn(&SR) -> Link,
            project: impl FnMut(&FR, &SR) -> Value,
        ) -> Result<Vec<WorthQueryManagedDerivedViewKey>, Denial>
        where
            FR: WorthQueryApplicationProjection<Schema, FQ> + 'a,
            SR: WorthQueryApplicationProjection<Schema, SQ> + 'a,
            Member: WorthQueryManagedDerivedMemberToken,
            Value: WorthQueryManagedDerivedValue,
            Link: Eq,
        {
            self.inner.reconstruct_managed_derived_collection_pair::<MQ, MR, Member, FQ, FP, FR, SQ, SP, SR, A, I, C, Value, Link>(request, view, product, membership_result, members, prepare, first_link, second_link, project)
        }
    }
}
