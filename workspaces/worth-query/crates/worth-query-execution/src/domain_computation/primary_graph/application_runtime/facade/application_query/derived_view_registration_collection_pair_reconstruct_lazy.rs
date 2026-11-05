//! The runtime's consumer methods over `derived_view_registration_collection_pair_reconstruct_lazy` in `domain_computation::primary_graph::application_query`; each only calls its subsystem.
use crate::basis::WorthQueryProductBranchLease;
use crate::domain_computation::primary_graph::application_query::derived_view::WorthQueryManagedDerivedValue;
use crate::domain_computation::primary_graph::application_query::derived_view::WorthQueryManagedDerivedView;
use crate::domain_computation::primary_graph::application_query::WorthQueryApplicationOneShotResult;
use crate::domain_computation::primary_graph::application_query::WorthQueryApplicationProjection;
use crate::domain_computation::primary_graph::application_query::WorthQueryManagedDerivedMemberToken;
use crate::domain_computation::primary_graph::application_query::WorthQueryManagedDerivedViewDenial;
use crate::domain_computation::primary_graph::application_query::WorthQueryManagedDerivedViewKey;
use crate::domain_computation::request_execution::WorthQueryAdvancementPhase;
use worth_query_installation::facade::ApplicationSchema;
use worth_relational::facade::identity::EntityId;

impl<Schema: ApplicationSchema>
    super::super::super::WorthQueryPrimaryGraphApplicationRuntime<Schema>
{
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
        first_for: impl FnMut(
            &Member,
        ) -> Result<
            WorthQueryApplicationOneShotResult<FQ, FR>,
            WorthQueryManagedDerivedViewDenial,
        >,
        second_for: impl FnMut(
            &FR,
        ) -> Result<
            WorthQueryApplicationOneShotResult<SQ, SR>,
            WorthQueryManagedDerivedViewDenial,
        >,
        first_link: impl Fn(&FR) -> Link,
        second_link: impl Fn(&SR) -> Link,
        project: impl FnMut(&FR, &SR) -> Value,
    ) -> Result<Vec<WorthQueryManagedDerivedViewKey>, WorthQueryManagedDerivedViewDenial>
    where
        FR: WorthQueryApplicationProjection<Schema, FQ>,
        SR: WorthQueryApplicationProjection<Schema, SQ>,
        Member: WorthQueryManagedDerivedMemberToken,
        Value: WorthQueryManagedDerivedValue,
        Link: Eq,
    {
        <Self as crate::domain_computation::primary_graph::application_query::ReconstructLazyOps<
            Schema,
        >>::reconstruct_managed_derived_collection_pair_lazy(
            self,
            phase,
            view,
            product,
            membership_result,
            members,
            first_for,
            second_for,
            first_link,
            second_link,
            project,
        )
    }
}
