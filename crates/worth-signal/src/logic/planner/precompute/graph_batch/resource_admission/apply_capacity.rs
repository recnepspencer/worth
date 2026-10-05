//! The graph owner admits the complete checked apply map before evaluator work.
//! Packet heap bounds come from the apply workspace's own field inventory.
use std::mem::size_of;

use crate::data::aspect::MAX_ASPECTS;
use crate::data::comparator::VersionComparatorPolicy;
use crate::data::dependency::DependencyEdge;
use crate::data::error::SignalError;
use crate::data::graph::SignalGraph;
use crate::data::handle::NodeId;
use crate::data::node::BoundedSignalInputs;
use crate::data::proof::invalidation::progression::GraphProposalKey;
use crate::data::retained_storage::RetainedStorageMeasurement;
use crate::data::retained_storage::RetainedStoragePreparation;
use crate::logic::planner::apply::workspace::{ApplyMemberBasis, GroupLocalApplyPacket};
use worth_execution::{ExecutionMap, ExecutionResourceLease};
use worth_foundational::PartitionIdentity;

pub(super) fn existing_edges_heap(
    current: &[DependencyEdge],
    work: &mut RetainedStoragePreparation<'_>,
) -> Result<u64, SignalError> {
    current.iter().try_fold(0_u64, |bytes, edge| {
        let edge_bytes = edge
            .retained_heap_charge(work)
            .map_err(|_| SignalError::EvaluationStorageCapacityExhausted)?
            .bytes();
        bytes
            .checked_add(edge_bytes)
            .ok_or(SignalError::EvaluationStorageCapacityExhausted)
    })
}

pub(super) fn measure_member(
    graph: &SignalGraph,
    node: NodeId,
    declaration: &BoundedSignalInputs,
    current: &[DependencyEdge],
    scope_heap: u64,
    old_edges_heap: u64,
    capture: u64,
    policy: VersionComparatorPolicy,
    work: &mut RetainedStoragePreparation<'_>,
) -> Result<ApplyMemberBasis, SignalError> {
    ApplyMemberBasis::measure(
        graph,
        node,
        declaration,
        current,
        scope_heap,
        old_edges_heap,
        capture,
        policy,
        work,
    )
}

/// One potential group per member is the largest generic framework shape.
/// Actual ExecutionMap::run remains the final owner of its exact reservation.
pub(super) fn map_memory_requirement(
    prior: &[ApplyMemberBasis],
    candidate: &ApplyMemberBasis,
    result_grant: u64,
    lease: &ExecutionResourceLease<'_>,
) -> Option<u64> {
    let width = prior.len().checked_add(1)?;
    let mut scratch = 0_u64;
    let mut result = 0_u64;
    for member in prior.iter().chain(std::iter::once(candidate)) {
        let (Some(member_scratch), Some(member_result)) = (
            member.scratch_bytes(result_grant).ok(),
            member.result_bytes(result_grant).ok(),
        ) else {
            return None;
        };
        let (Some(next_scratch), Some(next_result)) = (
            scratch.checked_add(member_scratch),
            result.checked_add(member_result),
        ) else {
            return None;
        };
        scratch = next_scratch;
        result = next_result;
    }
    // A singleton per selected member maximizes per-group structural slots.
    // Each write family has at most the sealed per-target effect-key breadth.
    let access_per_group = (MAX_ASPECTS + 7)
        .checked_mul(size_of::<GraphProposalKey>())
        .and_then(|bytes| bytes.checked_add(size_of::<Vec<GraphProposalKey>>()))
        .and_then(|bytes| {
            bytes.checked_add(size_of::<(PartitionIdentity, Vec<GraphProposalKey>)>())
        });
    let access = access_per_group
        .and_then(|bytes| bytes.checked_mul(width))
        .and_then(|bytes| u64::try_from(bytes).ok())?;
    ExecutionMap::<usize, GraphProposalKey>::declared_memory_requirement_for_lease::<
        GroupLocalApplyPacket,
        SignalError,
    >(lease, width, 0, scratch, result, access)
}
