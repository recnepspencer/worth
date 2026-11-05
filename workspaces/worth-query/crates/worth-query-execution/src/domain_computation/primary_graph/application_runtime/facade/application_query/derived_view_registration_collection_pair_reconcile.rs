//! The runtime's consumer methods over `derived_view_registration_collection_pair_reconcile` in `domain_computation::primary_graph::application_query`; each only calls its subsystem.
use crate::basis::WorthQueryProductBranchLease;
use crate::domain_computation::primary_graph::application_query::derived_view::WorthQueryManagedDerivedValue;
use crate::domain_computation::primary_graph::application_query::derived_view::WorthQueryManagedDerivedView;
use crate::domain_computation::primary_graph::application_query::WorthQueryApplicationOneShotResult;
use crate::domain_computation::primary_graph::application_query::WorthQueryApplicationProjection;
use crate::domain_computation::primary_graph::application_query::WorthQueryManagedDerivedMemberToken;
use crate::domain_computation::primary_graph::application_query::WorthQueryManagedDerivedViewDenial;
use crate::domain_computation::primary_graph::application_query::WorthQueryManagedDerivedViewReconciliation;
use worth_query_installation::facade::ApplicationSchema;
use worth_relational::facade::identity::EntityId;

impl<Schema: ApplicationSchema>
    super::super::super::WorthQueryPrimaryGraphApplicationRuntime<Schema>
{
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
    >(
        &self,
        view: &WorthQueryManagedDerivedView<MembershipQuery, Value>,
        product: &WorthQueryProductBranchLease,
        membership_result: &WorthQueryApplicationOneShotResult<MembershipQuery, MembershipResult>,
        members: impl Fn(&MembershipResult) -> Vec<(EntityId, Member)>,
        first_for: impl FnMut(
            &Member,
        ) -> Result<
            WorthQueryApplicationOneShotResult<FirstQuery, FirstResult>,
            WorthQueryManagedDerivedViewDenial,
        >,
        second_for: impl FnMut(
            &FirstResult,
        ) -> Result<
            WorthQueryApplicationOneShotResult<SecondQuery, SecondResult>,
            WorthQueryManagedDerivedViewDenial,
        >,
        first_link: impl Fn(&FirstResult) -> LinkKey,
        second_link: impl Fn(&SecondResult) -> LinkKey,
        project: impl FnMut(&FirstResult, &SecondResult) -> Value,
    ) -> Result<WorthQueryManagedDerivedViewReconciliation, WorthQueryManagedDerivedViewDenial>
    where
        FirstResult: WorthQueryApplicationProjection<Schema, FirstQuery>,
        SecondResult: WorthQueryApplicationProjection<Schema, SecondQuery>,
        Value: WorthQueryManagedDerivedValue,
        Member: WorthQueryManagedDerivedMemberToken,
        LinkKey: Eq,
    {
        <Self as crate::domain_computation::primary_graph::application_query::ReconcileOps<
            Schema,
        >>::reconcile_managed_derived_collection_pair_lazy(
            self,
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
