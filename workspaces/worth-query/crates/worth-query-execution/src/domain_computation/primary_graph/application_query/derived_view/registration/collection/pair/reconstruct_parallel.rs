//! Request-owned pair reads with canonical owner projection and atomic retention.

use super::{
    Denial, WorthQueryApplicationProjection, WorthQueryManagedDerivedValue,
    WorthQueryManagedDerivedView, WorthQueryManagedDerivedViewKey,
    WorthQueryPrimaryGraphApplicationRuntime,
};
use crate::basis::WorthQueryProductBranchLease;
use crate::domain_computation::primary_graph::application_query::derived_view::retention::{
    RetainedMemberToken, WorthQueryManagedDerivedMemberToken,
};
use crate::domain_computation::primary_graph::application_query::one_shot::outcome::finalize_one_shot;
#[cfg(test)]
use crate::domain_computation::primary_graph::application_query::DispatchWitness;
use crate::domain_computation::primary_graph::application_query::{
    WorthQueryApplicationAuthorizationWorkEvidence, WorthQueryApplicationOneShotResult,
    WorthQueryDerivedPairReadPlans,
};
use std::collections::BTreeMap;
use worth_execution::{
    ExecutionMap, ExecutionMemoryReservation, ExecutionRequest, KeylessPartition,
};
use worth_foundational::PartitionIdentity;
use worth_proof::CanonicalUniqueVec;
use worth_query_declaration::facade::application_schema::ApplicationSchema;
use worth_relational::facade::identity::EntityId;
mod canonical_results;
mod denial;
mod owner_stage;
mod preparation;

impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    /// Reconstruct unique roots under the caller's bounded request. Owner
    /// preparation precedes the owned map; application projection and all link
    /// checks run on the calling thread, in EntityId order. A completed compute
    /// prefix is projected only while the owner request checks permit it;
    /// cancellation or an elapsed deadline can stop its projection. Projection
    /// failures keep all work the map settled. Retention changes atomically
    /// only on success; admitting a newer product may first make the view cold.
    /// Reads serialize at the installed Relational owner: its native indexed
    /// read door still borrows the locked runtime. The map supplies canonical
    /// order, charging and one execution authority, not concurrent native reads.
    /// Cancellation and deadline boundaries may differ with placement: each
    /// reports a canonical prefix and only its admitted work, not width equality.
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
        mut prepare: impl FnMut(
            &Member,
        ) -> Result<
            WorthQueryDerivedPairReadPlans<'a, Schema, FQ, FP, FR, SQ, SP, SR, A, I, C>,
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
        request
            .run(
                worth_execution::ExecutionWorkCeiling::new(request.work_ceiling()),
                |lease| {
                    // Custody covers owner-owned plans and staging, separately from the
                    // map's transient input/output admission. It outlives all staged data.
                    let owner_bytes = u64::try_from(view.state.limits.maximum_retained_bytes())
                        .map_err(|_| Denial::CapacityOverflow)?;
                    let mut owner_hold =
                        ExecutionMemoryReservation::reserve_in_scope(lease, owner_bytes)
                            .map_err(denial::lease)?;
                    let (expected, membership) = owner_stage::run(lease, owner_bytes, |context| {
                        context
                            .checkpoint(1)
                            .map_err(|stop| denial::kernel(stop, None))?;
                        self.admit_view_product(view, product)?;
                        if membership_result.rows().len()
                            != membership_result.observed_sources().len()
                        {
                            return Err(Denial::IncompleteDependencies);
                        }
                        let mut membership = self.checked_view_dependencies(
                            view,
                            product,
                            membership_result.result_set_observation().source(),
                        )?;
                        let mut expected = BTreeMap::new();
                        for (row, source) in membership_result
                            .rows()
                            .iter()
                            .zip(membership_result.observed_sources())
                        {
                            context
                                .checkpoint(1)
                                .map_err(|stop| denial::kernel(stop, None))?;
                            membership
                                .extend(self.checked_view_dependencies(view, product, source)?);
                            for (root, member) in members(row) {
                                if expected.insert(root, member).is_some() {
                                    return Err(Denial::DuplicateRoot { root });
                                }
                                if expected.len() > view.state.limits.maximum_entries() {
                                    return Err(Denial::EntryCapacityExceeded);
                                }
                            }
                        }
                        Ok((expected, membership))
                    })?;
                    let roots =
                        CanonicalUniqueVec::from_btree_set(expected.keys().copied().collect());
                    let slots = std::mem::size_of::<
                        WorthQueryDerivedPairReadPlans<'a, Schema, FQ, FP, FR, SQ, SP, SR, A, I, C>,
                    >()
                    .checked_add(
                        2 * std::mem::size_of::<WorthQueryApplicationAuthorizationWorkEvidence>(),
                    )
                    .and_then(|bytes| bytes.checked_mul(expected.len()))
                    .and_then(|bytes| u64::try_from(bytes).ok())
                    .ok_or(Denial::ChargedBytesOverflow)?;
                    let plan_bytes = owner_bytes
                        .checked_add(slots)
                        .ok_or(Denial::ChargedBytesOverflow)?;
                    owner_hold.resize(plan_bytes).map_err(|memory| {
                        denial::lease(worth_execution::LeaseDenial::MemoryExhausted(memory))
                    })?;
                    let mut tokens = BTreeMap::new();
                    let mut plans = Vec::with_capacity(expected.len());
                    let mut authorization = Vec::with_capacity(expected.len());
                    owner_stage::run(lease, owner_bytes, |context| {
                        for (root, member) in expected {
                            context
                                .checkpoint(1)
                                .map_err(|stop| denial::kernel(stop, None))?;
                            let mut pair = prepare(&member)?;
                            if Some(pair.first.query_identity()) != view.state.entry_query.as_ref()
                                || Some(pair.second.query_identity())
                                    != view.state.secondary_entry_query.as_ref()
                            {
                                return Err(Denial::ForeignQuery);
                            }
                            let custody = pair.retained_input_bound()?;
                            // The map borrows these admitted plans; their heap custody
                            // stays here rather than being charged again as map input.
                            let input_bytes = owner_hold
                                .bytes()
                                .checked_add(custody)
                                .ok_or(Denial::ChargedBytesOverflow)?;
                            owner_hold.resize(input_bytes).map_err(|memory| {
                                denial::lease(worth_execution::LeaseDenial::MemoryExhausted(memory))
                            })?;
                            authorization.push(pair.admit(self, root, context)?);
                            tokens.insert(root, RetainedMemberToken::new(member));
                            plans.push(pair);
                        }
                        Ok(())
                    })?;
                    let partitions = plans
                        .iter()
                        .zip(roots.as_slice())
                        .enumerate()
                        .map(|(index, (pair, root))| {
                            let capacity = pair.result_capacity()?;
                            Ok((
                                PartitionIdentity::new(index as u64 + 1),
                                KeylessPartition {
                                    value: pair.worker(self, *root)?,
                                    kernel_scratch_bytes: 0,
                                    max_result_bytes: capacity,
                                },
                            ))
                        })
                        .collect::<Result<BTreeMap<_, _>, Denial>>()?;
                    let map = ExecutionMap::<_, ()>::from_keyless_partitions(partitions).map_err(
                        |worth_execution::MapMemoryOverflow| Denial::ChargedBytesOverflow,
                    )?;
                    let (outputs, stop) = canonical_results::CanonicalPairResults::settle(
                        roots,
                        map.run_owned(lease, |pair, context| pair.run(context)),
                    );
                    // The map releases its result allowance at settlement. Admit the
                    // returned prefix for owner staging before projection; these two
                    // phases never reserve the same result allowance simultaneously.
                    let staged_bytes =
                        plans
                            .iter()
                            .take(outputs.len())
                            .try_fold(0_u64, |bytes, pair| {
                                bytes
                                    .checked_add(pair.result_capacity()?)
                                    .ok_or(Denial::ChargedBytesOverflow)
                            })?;
                    let output_bytes = owner_hold
                        .bytes()
                        .checked_add(staged_bytes)
                        .ok_or(Denial::ChargedBytesOverflow)?;
                    owner_hold.resize(output_bytes).map_err(|memory| {
                        denial::lease(worth_execution::LeaseDenial::MemoryExhausted(memory))
                    })?;
                    // Only this private canonical pairing can reach owner application.
                    let mut keys = Vec::with_capacity(outputs.len());
                    let mut entries = Vec::with_capacity(outputs.len());
                    owner_stage::run(lease, owner_bytes, |context| {
                        for (((root, output), pair), (first_work, second_work)) in
                            outputs.into_prefix().zip(plans).zip(authorization)
                        {
                            context
                                .checkpoint(
                                    pair.projection_work()?
                                        .checked_add(1)
                                        .ok_or(Denial::WorkCounterOverflow)?,
                                )
                                .map_err(|stop| denial::kernel(stop, Some(root)))?;
                            let first_proof = pair
                                .first
                                .graph_work
                                .seal_prepared_read(output.first.proof)
                                .map_err(denial::session)?;
                            let second_proof = pair
                                .second
                                .graph_work
                                .seal_prepared_read(output.second.proof)
                                .map_err(denial::session)?;
                            let first = finalize_one_shot(
                                self,
                                pair.first,
                                output.first.raw,
                                first_work,
                                first_proof,
                                None,
                            )
                            .map_err(|denial| Denial::ReadDenied { root, denial })?;
                            let second = finalize_one_shot(
                                self,
                                pair.second,
                                output.second.raw,
                                second_work,
                                second_proof,
                                None,
                            )
                            .map_err(|denial| Denial::ReadDenied { root, denial })?;
                            let (key, value, dependencies) = self
                                .read_managed_entry_pair_from_result(
                                    view,
                                    product,
                                    root,
                                    first,
                                    |_| Ok(second),
                                    &first_link,
                                    &second_link,
                                    |first, second| project(first, second),
                                )?;
                            #[cfg(test)]
                            DispatchWitness::applied(root, &dependencies);
                            keys.push(key.clone());
                            entries.push((key, value, dependencies));
                        }
                        Ok(())
                    })?;
                    if let Some(stop) = stop {
                        return Err(stop);
                    }
                    owner_stage::run(lease, owner_bytes, |context| {
                        context
                            .checkpoint(1)
                            .map_err(|stop| denial::kernel(stop, None))?;
                        self.admit_view_product(view, product)?;
                        view.state.reconstruct(
                            membership_result
                                .result_set_observation()
                                .source()
                                .managed_derived_view_key(),
                            Some(tokens),
                            entries,
                            membership,
                            product.selected_commit(),
                        )
                    })?;
                    Ok(keys)
                },
            )
            .map_err(denial::scope)?
            .0
    }
}

#[cfg(test)]
mod denial_tests;

#[cfg(test)]
mod read_ceiling_tests;
