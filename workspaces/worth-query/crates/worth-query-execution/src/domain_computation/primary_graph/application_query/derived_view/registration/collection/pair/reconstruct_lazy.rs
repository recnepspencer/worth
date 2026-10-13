//! Sequential reconstruction when the first genuine row selects the second scope.

use super::{
    reconstruction::{denial, owner_stage},
    Denial, WorthQueryApplicationOneShotResult, WorthQueryApplicationProjection,
    WorthQueryManagedDerivedValue, WorthQueryManagedDerivedView, WorthQueryManagedDerivedViewKey,
    WorthQueryPrimaryGraphApplicationRuntime,
};
use crate::basis::WorthQueryProductBranchLease;
use crate::domain_computation::primary_graph::{
    application_query::derived_view::retention::{
        RetainedMemberToken, WorthQueryManagedDerivedMemberToken,
    },
    WorthQueryAdvancementPhase,
};
use std::collections::BTreeMap;
use worth_execution::{ExecutionMemoryReservation, ExecutionWorkCeiling};
use worth_query_declaration::facade::application_schema::ApplicationSchema;
use worth_relational::facade::identity::EntityId;

impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    /// Reads in canonical root order under the caller's actual advancement.
    /// The second read may depend on the first row and use a distinct root.
    /// Both callbacks retain their ordinary query authorization and source
    /// checks. Callers keep any shared-batch row claims through projection.
    /// Failure publishes no partial image; successful retention is atomic.
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
        mut first_for: impl FnMut(&Member) -> Result<WorthQueryApplicationOneShotResult<FQ, FR>, Denial>,
        mut second_for: impl FnMut(&FR) -> Result<WorthQueryApplicationOneShotResult<SQ, SR>, Denial>,
        first_link: impl Fn(&FR) -> Link,
        second_link: impl Fn(&SR) -> Link,
        mut project: impl FnMut(&FR, &SR) -> Value,
    ) -> Result<Vec<WorthQueryManagedDerivedViewKey>, Denial>
    where
        FR: WorthQueryApplicationProjection<Schema, FQ>,
        SR: WorthQueryApplicationProjection<Schema, SQ>,
        Member: WorthQueryManagedDerivedMemberToken,
        Value: WorthQueryManagedDerivedValue,
        Link: Eq,
    {
        let request = phase
            .execution_request_for(&self.product_runtime)
            .map_err(|_| Denial::ForeignApplication)?;
        request
            .run(ExecutionWorkCeiling::new(request.work_ceiling()), |lease| {
                let result = (|| {
                    let mut hold = ExecutionMemoryReservation::reserve_in_scope(lease, 0)
                        .map_err(denial::lease)?;
                    let (expected, membership) = super::reconstruction::membership::prepare(
                        self, lease, &mut hold, view, product, membership_result, members,
                    )?;
                    owner_stage::reserve_entry_slots::<WorthQueryManagedDerivedViewKey, Value>(
                        &mut hold, expected.len(),
                    )?;
                    let mut tokens = BTreeMap::new();
                    let mut keys = Vec::with_capacity(expected.len());
                    let mut entries = Vec::with_capacity(expected.len());
                    for (root, member) in expected {
                        owner_stage::run(lease, |context| {
                            context.checkpoint(1).map_err(|stop| denial::kernel(stop, Some(root)))?;
                            owner_stage::retain_member_token(&mut hold, &member)?;
                            tokens.insert(root, RetainedMemberToken::clone_admitted(&member)?);
                            Ok(())
                        })?;
                        let first = first_for(&member)?;
                        let entry = self.read_managed_entry_pair_from_result(
                            view, product, root, first, |row| second_for(row),
                            &first_link, &second_link, |first, second| project(first, second),
                        )?;
                        owner_stage::run(lease, |context| {
                            context.checkpoint(1).map_err(|stop| denial::kernel(stop, Some(root)))?;
                            owner_stage::retain_entry_payload(&mut hold, &entry.0, &entry.1, &entry.2)?;
                            keys.push(entry.0.clone());
                            entries.push(entry);
                            Ok(())
                        })?;
                    }
                    owner_stage::run(lease, |context| {
                        context.checkpoint(1).map_err(|stop| denial::kernel(stop, None))?;
                        self.admit_view_product(view, product)?;
                        owner_stage::retain_image(&mut hold, &entries, &membership, &tokens)?;
                        view.state.reconstruct(
                            membership_result.result_set_observation().source().managed_derived_view_key(),
                            Some(tokens), entries, membership, product.selected_commit(),
                        )
                    })?;
                    Ok(keys)
                })();
                crate::domain_computation::primary_graph::application_query::one_shot::retain_reconstruction_result(lease, result, denial::lease)
            })
            .map_err(denial::scope)?
            .0
    }
}
