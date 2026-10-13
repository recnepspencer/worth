//! Serial runtime-dependent planning on one request and native read loan.

use super::{
    reconstruction::owner_stage as stage, reconstruction_denial as denial, Denial,
    WorthQueryApplicationProjection, WorthQueryManagedDerivedValue, WorthQueryManagedDerivedView,
    WorthQueryManagedDerivedViewKey, WorthQueryPrimaryGraphApplicationRuntime,
};
use crate::basis::WorthQueryProductBranchLease;
use crate::domain_computation::primary_graph::application_query::{
    derived_view::retention::{RetainedMemberToken, WorthQueryManagedDerivedMemberToken},
    WorthQueryApplicationOneShotResult, WorthQueryApplicationQueryBatchAdmission,
    WorthQueryDependentPairReadContext,
};
use std::collections::BTreeMap;
use worth_execution::{ExecutionMemoryReservation, ExecutionRequest, ExecutionWorkCeiling};
use worth_query_declaration::facade::application_schema::ApplicationSchema;
use worth_relational::facade::identity::EntityId;

impl<S: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<S> {
    /// Cold reconstruction when the first row supplies the second read's scope.
    /// Inside host advancement, borrow the request from the issued phase's
    /// `checked_execution_request_for` or narrow its active lease with a genuine
    /// child. An independently created request has unrelated nested custody.
    /// Both callbacks resolve local scopes and execute admitted plans through
    /// the supplied Query context on the calling thread,
    /// under this request and the same genuine loan. They cannot dispatch reads
    /// to workers. Each pair's claims survive source/link checks and projection.
    /// Roots are visited in EntityId order and the retained image changes only
    /// after every pair succeeds and the product is still current. A newer
    /// product can make the previous image cold before reconstruction finishes.
    ///
    /// Cost is the membership walk plus both ordinary reads per member and the
    /// projected image. This serial dependency makes no worker parity claim.
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
        mut first_for: impl FnMut(
            &Member,
            &mut WorthQueryDependentPairReadContext<'_, '_, '_, '_, '_, S, FQ, FR>,
        ) -> Result<(), Denial>,
        mut second_for: impl FnMut(
            &FR,
            &mut WorthQueryDependentPairReadContext<'_, '_, '_, '_, '_, S, SQ, SR>,
        ) -> Result<(), Denial>,
        first_link: impl Fn(&FR) -> Link,
        second_link: impl Fn(&SR) -> Link,
        mut project: impl FnMut(&FR, &SR) -> Value,
    ) -> Result<Vec<WorthQueryManagedDerivedViewKey>, Denial>
    where
        FR: WorthQueryApplicationProjection<S, FQ>,
        SR: WorthQueryApplicationProjection<S, SQ>,
        Member: WorthQueryManagedDerivedMemberToken,
        Value: WorthQueryManagedDerivedValue,
        Link: Eq,
    {
        request.run(ExecutionWorkCeiling::new(request.work_ceiling()), |lease| {
            let result = (|| {
                let mut hold = ExecutionMemoryReservation::reserve_in_scope(lease, 0)
                    .map_err(denial::lease)?;
                let (expected, membership) = super::reconstruction::membership::prepare(
                    self, lease, &mut hold, view, product, membership_result, members,
                )?;
                let slot_bytes = std::mem::size_of::<(
                    WorthQueryManagedDerivedViewKey, Value,
                    std::collections::BTreeSet<crate::domain_computation::primary_graph::application_query::derived_view::dependency::ViewDependency>,
                )>().checked_add(std::mem::size_of::<WorthQueryManagedDerivedViewKey>())
                    .and_then(|bytes| bytes.checked_mul(expected.len()))
                    .and_then(|bytes| u64::try_from(bytes).ok())
                    .ok_or(Denial::ChargedBytesOverflow)?;
                let _slots = ExecutionMemoryReservation::reserve_in_scope(lease, slot_bytes)
                    .map_err(denial::lease)?;
                let mut tokens = BTreeMap::new();
                let mut entries = Vec::with_capacity(expected.len());
                let mut keys = Vec::with_capacity(expected.len());
                for (root, member) in expected {
                    let (first, first_scope) = stage::run(lease, |context| {
                        context.checkpoint(1).map_err(|stop| denial::kernel(stop, Some(root)))?;
                        let mut read = WorthQueryDependentPairReadContext {
                            application: self, request: &request, product,
                            query: view.state.entry_query.as_ref().ok_or(Denial::ForeignQuery)?,
                            root, scope_root: Some(root), batch, lease, context,
                            request_scope: None, attempted: false, result: None,
                        };
                        first_for(&member, &mut read)?;
                        Ok((read.result.ok_or(Denial::QueryExecutionDenied)?, read.request_scope))
                    })?;
                    let (first_result, _first_claim) = first.into_parts();
                    let mut second_claim = None;
                    let (key, value, dependencies) = stage::run(lease, |context| {
                        self.read_managed_entry_pair_from_result(
                            view, product, root, first_result,
                            |row| {
                                context.checkpoint(1).map_err(|stop| denial::kernel(stop, Some(root)))?;
                                let mut read = WorthQueryDependentPairReadContext {
                                    application: self, request: &request, product,
                                    query: view.state.secondary_entry_query.as_ref().ok_or(Denial::ForeignQuery)?,
                                    root, scope_root: None, batch, lease, context,
                                    request_scope: first_scope, attempted: false, result: None,
                                };
                                second_for(row, &mut read)?;
                                let (result, claim) = read.result.ok_or(Denial::QueryExecutionDenied)?.into_parts();
                                context.checkpoint(1).map_err(|stop| denial::kernel(stop, Some(root)))?;
                                second_claim = Some(claim);
                                Ok(result)
                            },
                            &first_link, &second_link, |first, second| project(first, second),
                        )
                    })?;
                    stage::retain_entry(&mut hold, &key, &value, &dependencies)?;
                    stage::retain_member_token(&mut hold, &member)?;
                    tokens.insert(root, RetainedMemberToken::clone_admitted(&member)?);
                    keys.push(key.clone());
                    entries.push((key, value, dependencies));
                }
                stage::run(lease, |context| {
                    context.checkpoint(1).map_err(|stop| denial::kernel(stop, None))?;
                    self.admit_view_product(view, product)?;
                    stage::retain_image(&mut hold, &entries, &membership, &tokens)?;
                    view.state.reconstruct(
                        membership_result.result_set_observation().source().managed_derived_view_key(),
                        Some(tokens), entries, membership, product.selected_commit(),
                    )
                })?;
                Ok(keys)
            })();
            crate::domain_computation::primary_graph::application_query::one_shot::retain_reconstruction_result(
                lease, result, denial::lease,
            )
        }).map_err(denial::scope)?.0
    }
}
