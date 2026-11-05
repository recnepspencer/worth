//! The runtime's consumer methods over `derived_view_registration_collection_pair_reconstruct_dependent` in `domain_computation::primary_graph::application_query`; each only calls its subsystem.
use crate::basis::WorthQueryProductBranchLease;
use crate::domain_computation::primary_graph::application_query::derived_view::WorthQueryManagedDerivedValue;
use crate::domain_computation::primary_graph::application_query::derived_view::WorthQueryManagedDerivedView;
use crate::domain_computation::primary_graph::application_query::one_shot::*;
use crate::domain_computation::primary_graph::application_query::WorthQueryApplicationOneShotResult;
use crate::domain_computation::primary_graph::application_query::WorthQueryApplicationProjection;
use crate::domain_computation::primary_graph::application_query::WorthQueryApplicationQueryBatchAdmission;
use crate::domain_computation::primary_graph::application_query::WorthQueryManagedDerivedMemberToken;
use crate::domain_computation::primary_graph::application_query::WorthQueryManagedDerivedViewDenial;
use crate::domain_computation::primary_graph::application_query::WorthQueryManagedDerivedViewKey;
use worth_execution::ExecutionRequest;
use worth_query_installation::facade::ApplicationSchema;
use worth_relational::facade::identity::EntityId;

impl<S: ApplicationSchema> super::super::super::WorthQueryPrimaryGraphApplicationRuntime<S> {
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
        ) -> Result<(), WorthQueryManagedDerivedViewDenial>,
        second_for: impl FnMut(
            &FR,
            &mut WorthQueryDependentPairReadContext<'_, '_, '_, '_, '_, S, SQ, SR>,
        ) -> Result<(), WorthQueryManagedDerivedViewDenial>,
        first_link: impl Fn(&FR) -> Link,
        second_link: impl Fn(&SR) -> Link,
        project: impl FnMut(&FR, &SR) -> Value,
    ) -> Result<Vec<WorthQueryManagedDerivedViewKey>, WorthQueryManagedDerivedViewDenial>
    where
        FR: WorthQueryApplicationProjection<S, FQ>,
        SR: WorthQueryApplicationProjection<S, SQ>,
        Member: WorthQueryManagedDerivedMemberToken,
        Value: WorthQueryManagedDerivedValue,
        Link: Eq,
    {
        <Self as crate::domain_computation::primary_graph::application_query::ReconstructDependentOps<S>>::reconstruct_managed_derived_collection_pair_dependent_in_batch(self, request, view, product, membership_result, members, batch, first_for, second_for, first_link, second_link, project)
    }
}
