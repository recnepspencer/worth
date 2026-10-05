//! Candidate map capacity comes from the indexed membership owner and the
//! admitted checked output envelope, before any evaluator runs.

use std::mem::size_of;

use worth_execution::{ExecutionMap, ExecutionResourceLease, MapKernelContext, MapPartition};
use worth_foundational::PartitionIdentity;

use crate::data::aspect::{Aspect, AspectMask, MAX_ASPECTS};
use crate::data::error::SignalError;
use crate::data::graph::SignalGraph;
use crate::data::handle::NodeId;
use crate::data::output::{ChangedRegion, ScopePath};
use crate::data::proof::invalidation::output_commit::{ProducedAspectChange, ScopePrecision};
use crate::data::retained_storage::ordered_lookup_steps;

use super::prepared::CandidateTask;
use super::{
    ProducerAspectKey, ReverseSubscriptionIndex, ReverseSubscriptionQuery,
    ReverseSubscriptionStorage,
};

/// Fixed-domain membership facts read once before selecting a result grant.
pub(crate) struct CandidateEpochBasis {
    members: [Option<usize>; MAX_ASPECTS],
}

impl CandidateEpochBasis {
    pub(crate) fn measure_member(
        graph: &SignalGraph,
        producer: NodeId,
        work: &mut MapKernelContext<'_, '_>,
    ) -> Result<Self, SignalError> {
        if !graph.topology.reverse_subscriptions.is_valid() {
            return Err(SignalError::internal(
                "reverse subscription index requires authority rebuild",
            ));
        }
        let produces = graph.get_contract(producer)?.semantics.produces;
        let mut members = [None; MAX_ASPECTS];
        for (index, slot) in members.iter_mut().enumerate() {
            let aspect = Aspect::new(index as u8);
            if produces.contains(AspectMask::from_aspect(aspect)) {
                *slot = Some(
                    graph
                        .topology
                        .reverse_subscriptions
                        .candidate_member_count(producer, aspect, work)?,
                );
            }
        }
        Ok(Self { members })
    }

    fn capacities(
        &self,
        grant: u64,
    ) -> impl Iterator<Item = Result<CandidateCapacity, SignalError>> + '_ {
        self.members
            .iter()
            .flatten()
            .map(move |&members| CandidateCapacity::from_members(members, grant))
    }

    pub(crate) fn preparation_bytes(&self) -> Option<u64> {
        let count = self.members.iter().flatten().count();
        let inline = [
            size_of::<(NodeId, Aspect)>(),
            size_of::<MapPartition<CandidateTask, u8>>(),
            size_of::<PartitionIdentity>(),
            size_of::<bool>(),
            size_of::<Option<(&ProducedAspectChange, ScopePrecision)>>(),
            size_of::<ReverseSubscriptionQuery>() * 2,
            size_of::<u8>(),
            size_of::<(PartitionIdentity, Vec<u8>)>(),
            size_of::<Vec<u8>>(),
            size_of::<(PartitionIdentity, CandidateTask, u64, u64)>(),
        ]
        .into_iter()
        .try_fold(0_usize, usize::checked_add)?;
        let members = self
            .members
            .iter()
            .flatten()
            .try_fold(0_u64, |total, &members| {
                total.checked_add(
                    u64::try_from(members)
                        .ok()?
                        .checked_mul(size_of::<NodeId>() as u64)?,
                )
            })?;
        u64::try_from(count.checked_mul(inline)?)
            .ok()?
            .checked_add(members)
    }
}

/// Read-only requirement for the exact candidate map partition shape. The
/// supplied lease must be the child lease that will own its prepared ticket.
pub(crate) fn candidate_map_memory_requirement(
    prior: &[CandidateEpochBasis],
    candidate: &CandidateEpochBasis,
    output_heap_grant: u64,
    lease: &ExecutionResourceLease<'_>,
) -> Option<u64> {
    let mut count = 0_usize;
    let mut scratch = 0_u64;
    let mut result = 0_u64;
    for basis in prior.iter().chain(std::iter::once(candidate)) {
        for capacity in basis.capacities(output_heap_grant) {
            let capacity = capacity.ok()?;
            count = count.checked_add(1)?;
            scratch = scratch.checked_add(capacity.scratch_bytes)?;
            result = result.checked_add(capacity.result_bytes)?;
        }
    }
    let access_per_key = size_of::<u8>()
        .checked_add(size_of::<Vec<u8>>())?
        .checked_add(size_of::<(PartitionIdentity, Vec<u8>)>())?;
    let access = u64::try_from(access_per_key.checked_mul(count)?).ok()?;
    ExecutionMap::<CandidateTask, u8>::declared_memory_requirement_for_lease::<
        ReverseSubscriptionQuery,
        SignalError,
    >(lease, count, 0, scratch, result, access)
}

pub(super) struct CandidateCapacity {
    pub(super) scratch_bytes: u64,
    pub(super) result_bytes: u64,
}

impl ReverseSubscriptionIndex {
    fn candidate_member_count(
        &self,
        producer: NodeId,
        aspect: Aspect,
        work: &mut MapKernelContext<'_, '_>,
    ) -> Result<usize, SignalError> {
        let key = ProducerAspectKey::from_committed_output(producer, aspect);
        Ok(match &self.storage {
            ReverseSubscriptionStorage::Exclusive(flat) => {
                work.checkpoint(ordered_lookup_steps(flat.buckets.len()) as u64)
                    .map_err(|_| SignalError::invalid_input("candidate capacity lookup stopped"))?;
                flat.buckets.get(&key).map_or(0, |bucket| bucket.all.len())
            }
            ReverseSubscriptionStorage::ForkShared {
                base,
                bucket_changes,
                ..
            } => {
                work.checkpoint(
                    ordered_lookup_steps(base.buckets.len())
                        .saturating_add(ordered_lookup_steps(bucket_changes.len()))
                        as u64,
                )
                .map_err(|_| SignalError::invalid_input("candidate capacity lookup stopped"))?;
                base.buckets
                    .get(&key)
                    .map_or(0, |bucket| bucket.all.len())
                    .checked_add(
                        bucket_changes
                            .get(&key)
                            .map_or(0, |bucket| bucket.all.added_count()),
                    )
                    .ok_or_else(overflow)?
            }
        })
    }

    pub(super) fn candidate_capacity(
        &self,
        producer: NodeId,
        aspect: Aspect,
        output_heap_grant: u64,
        work: &mut MapKernelContext<'_, '_>,
    ) -> Result<CandidateCapacity, SignalError> {
        CandidateCapacity::from_members(
            self.candidate_member_count(producer, aspect, work)?,
            output_heap_grant,
        )
    }
}

impl CandidateCapacity {
    fn from_members(possible_members: usize, output_heap_grant: u64) -> Result<Self, SignalError> {
        // ProducedAspectDelta copies only the two region vectors returned by
        // NodeEvaluationResult. Every possible scope occupies at least one
        // ChangedRegion slot in the admitted noncapture result heap. When no
        // such slot fits, discovery necessarily takes the whole-aspect path.
        // Otherwise each scope can read the unscoped, ancestor and matching
        // buckets. The all-members bucket bounds each of those reads without
        // walking the reverse index's unrelated aspects or consumers.
        let possible_scopes = output_heap_grant
            / u64::try_from(size_of::<ChangedRegion>()).map_err(|_| overflow())?;
        let groups_per_scope = u64::try_from(ScopePath::MAX_DEPTH + 2).map_err(|_| overflow())?;
        // One incoming bucket can coexist with the accumulated Vec. After
        // normalization its exact-capacity result copy can coexist with the
        // old Vec. Both cases need one extra all-members allowance.
        let scratch_groups = if possible_scopes == 0 {
            2
        } else {
            possible_scopes
                .checked_mul(groups_per_scope)
                .and_then(|count| count.checked_add(1))
                .ok_or_else(overflow)?
        };
        let member_bytes = u64::try_from(possible_members)
            .ok()
            .and_then(|count| count.checked_mul(size_of::<NodeId>() as u64))
            .ok_or_else(overflow)?;
        Ok(Self {
            scratch_bytes: member_bytes
                .checked_mul(scratch_groups)
                .ok_or_else(overflow)?,
            result_bytes: member_bytes,
        })
    }
}

fn overflow() -> SignalError {
    SignalError::invalid_input("reverse candidate capacity overflow")
}
