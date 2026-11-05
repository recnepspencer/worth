//! The runtime's consumer methods over `derived_view_registration_collection_pair_reconstruct_parallel` in `domain_computation::primary_graph::application_query`; each only calls its subsystem.
use crate::basis::WorthQueryProductBranchLease;
use crate::domain_computation::primary_graph::application_query::derived_view::WorthQueryManagedDerivedValue;
use crate::domain_computation::primary_graph::application_query::derived_view::WorthQueryManagedDerivedView;
use crate::domain_computation::primary_graph::application_query::one_shot::*;
use crate::domain_computation::primary_graph::application_query::WorthQueryApplicationOneShotResult;
use crate::domain_computation::primary_graph::application_query::WorthQueryApplicationProjection;
use crate::domain_computation::primary_graph::application_query::WorthQueryManagedDerivedMemberToken;
use crate::domain_computation::primary_graph::application_query::WorthQueryManagedDerivedViewDenial;
use crate::domain_computation::primary_graph::application_query::WorthQueryManagedDerivedViewKey;
use worth_execution::ExecutionRequest;
use worth_query_installation::facade::ApplicationSchema;
use worth_relational::facade::identity::EntityId;

impl<Schema: ApplicationSchema>
    super::super::super::WorthQueryPrimaryGraphApplicationRuntime<Schema>
{
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
            WorthQueryManagedDerivedViewDenial,
        >,
        first_link: impl Fn(&FR) -> Link,
        second_link: impl Fn(&SR) -> Link,
        project: impl FnMut(&FR, &SR) -> Value,
    ) -> Result<Vec<WorthQueryManagedDerivedViewKey>, WorthQueryManagedDerivedViewDenial>
    where
        FR: WorthQueryApplicationProjection<Schema, FQ> + 'a,
        SR: WorthQueryApplicationProjection<Schema, SQ> + 'a,
        Member: WorthQueryManagedDerivedMemberToken,
        Value: WorthQueryManagedDerivedValue,
        Link: Eq,
    {
        <Self as crate::domain_computation::primary_graph::application_query::ReconstructParallelOps<Schema>>::reconstruct_managed_derived_collection_pair(self, request, view, product, membership_result, members, prepare, first_link, second_link, project)
    }
}
