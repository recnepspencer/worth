//! Row-only projection delegates to the same request-owned reconstruction.
use super::*;

impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    /// Reconstruct paired reads with row-only owner projection.
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
        SC: 'a,
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
            WorthQueryDerivedPairReadPlans<'a, Schema, FQ, FP, FR, SQ, SP, SR, A, I, C, SC>,
            Denial,
        >,
        first_link: impl Fn(&FR) -> Link,
        second_link: impl Fn(&SR) -> Link,
        mut project: impl FnMut(&FR, &SR) -> Value,
    ) -> Result<Vec<WorthQueryManagedDerivedViewKey>, Denial>
    where
        FR: WorthQueryApplicationProjection<Schema, FQ> + 'a,
        SR: WorthQueryApplicationProjection<Schema, SQ> + 'a,
        Member: WorthQueryManagedDerivedMemberToken,
        Value: WorthQueryManagedDerivedValue,
        Link: Eq,
    {
        self.reconstruct_managed_derived_collection_pair_with_receipts(
            request,
            view,
            product,
            membership_result,
            members,
            prepare,
            first_link,
            second_link,
            |first, second, _, _| Ok(project(first, second)),
        )
    }
}
