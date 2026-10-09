//! Fixed graph preparation shape and the common result allowance.
use crate::data::aspect::{PartitionVersionOverrides, MAX_ASPECTS};
use crate::data::error::SignalError;
use crate::data::graph::storage::invalidation_causes::ObservedCauseScope;
use crate::data::graph::{candidate_map_memory_requirement, CandidateEpochBasis};
use crate::data::handle::NodeId;
use crate::data::node::DeclaredSignalInput;
use crate::data::proof::invalidation::progression::{GraphMemberBinding, GraphProposalKey};
use crate::logic::planner::precompute::eligibility::PrevalidatedTask;
use crate::logic::planner::precompute::read_preparation::{
    checked_map_memory_requirement, PrecomputeMapBasis,
};
use crate::logic::prepared::PreparedEvaluation;
use worth_execution::{ExecutionResourceLease, MapKernelContext};

use super::{
    apply_capacity, bytes, cause_capacity::AcceptedCauseScope, overflow, ApplyMemberBasis,
    ApplyMemberCapacity,
};

pub(super) struct CandidateMetadata<'a> {
    pub edge_count: u64,
    pub scope_heap: u64,
    pub capture: u64,
    pub old_edges_heap: u64,
    pub dependency_set: u64,
    pub selected_heap: u64,
    pub selected_index: u64,
    pub producer_index: u64,
    pub consumer_index: u64,
    pub waiter_index: u64,
    pub waiter_scratch: u64,
    pub source_index: u64,
    pub subscriber_storage: u64,
    pub cause_growth: u64,
    pub diagnostic_fixed: u64,
    pub cause_scope_index: u64,
    pub owner_shape_growth: u64,
    pub node_root_growth: u64,
    pub node_capacity: usize,
    pub source_capacity: usize,
    pub sources_len: usize,
    pub fanout_len: usize,
    pub basis_growth_capacity: usize,
    pub apply_basis: &'a ApplyMemberBasis,
    pub candidate_preparation: u64,
}

impl CandidateMetadata<'_> {
    pub(super) fn fixed_bytes(self) -> Result<u64, SignalError> {
        let edge_copy = bytes::<DeclaredSignalInput>(self.edge_count)?
            .checked_add(self.scope_heap)
            .ok_or_else(overflow)?;
        let fixed = bytes::<PrevalidatedTask>(1)?
            .checked_add(bytes::<GraphMemberBinding>(2)?)
            .and_then(|total| total.checked_add(bytes::<PreparedEvaluation>(1).ok()?))
            .and_then(|total| {
                total.checked_add(bytes::<GraphProposalKey>((MAX_ASPECTS * 2 + 16) as u64).ok()?)
            })
            .and_then(|total| {
                total.checked_add(bytes::<u64>(self.edge_count.checked_mul(2)?).ok()?)
            })
            .and_then(|bytes| bytes.checked_add(edge_copy.checked_mul(3)?))
            .and_then(|bytes| bytes.checked_add(self.capture.checked_mul(3)?))
            .and_then(|bytes| bytes.checked_add(self.old_edges_heap.checked_mul(2)?))
            .ok_or_else(overflow)?;
        let selected = [
            self.dependency_set,
            self.selected_heap,
            self.selected_index,
            self.producer_index,
            self.consumer_index,
            self.waiter_index,
            self.waiter_scratch,
            self.source_index,
            self.subscriber_storage,
            self.cause_growth,
            self.diagnostic_fixed,
            self.cause_scope_index,
            self.owner_shape_growth,
            self.node_root_growth,
            self.candidate_preparation,
        ]
        .into_iter()
        .try_fold(fixed, |bytes, part| {
            bytes.checked_add(part).ok_or_else(overflow)
        })?;
        selected
            .checked_add(bytes::<NodeId>(
                self.node_capacity.saturating_add(self.source_capacity) as u64,
            )?)
            .and_then(|total| {
                total.checked_add(bytes::<(NodeId, usize, u64)>(self.sources_len as u64).ok()?)
            })
            .and_then(|total| {
                total
                    .checked_add(bytes::<ObservedCauseScope>(self.fanout_len as u64).ok()?)
                    .and_then(|total| {
                        total.checked_add(bytes::<AcceptedCauseScope>(self.fanout_len as u64).ok()?)
                    })
            })
            .and_then(|total| {
                total
                    .checked_add(bytes::<ApplyMemberBasis>(self.basis_growth_capacity as u64).ok()?)
            })
            .and_then(|total| {
                total.checked_add(
                    bytes::<CandidateEpochBasis>(self.basis_growth_capacity as u64).ok()?,
                )
            })
            .and_then(|total| {
                total.checked_add(
                    bytes::<PrecomputeMapBasis>(self.basis_growth_capacity as u64).ok()?,
                )
            })
            .and_then(|total| total.checked_add(bytes::<ApplyMemberCapacity>(1).ok()?))
            .and_then(|bytes| bytes.checked_add(self.apply_basis.admission_bytes().ok()?))
            .ok_or_else(overflow)
    }
}

pub(super) fn result_copies(fanout: usize) -> Result<u64, SignalError> {
    // Lowered value, worker input, pending snapshot, hot/cold scopes, delta,
    // semantic image, two semantic/fact capacity envelopes and snapshot shape.
    const FIXED: u64 = 1 + 1 + 1 + 2 + 1 + 1 + 32 + 1;
    (MAX_ASPECTS as u64)
        .checked_add(fanout as u64)
        .and_then(|copies| copies.checked_add(FIXED))
        .ok_or_else(overflow)
}

pub(super) struct CapacityDenial {
    pub required: u64,
    pub reserved: u64,
}

pub(super) fn choose_grant(
    available: u64,
    fixed: u64,
    copies: u64,
    width: usize,
    declared_result_minimum: u64,
    lease: worth_execution::ExecutionRequest<'_, '_>,
    candidate_lease: Option<&ExecutionResourceLease<'_>>,
    bases: &[ApplyMemberBasis],
    basis: &ApplyMemberBasis,
    candidate_bases: &[CandidateEpochBasis],
    candidate_basis: &CandidateEpochBasis,
    precompute_bases: &[PrecomputeMapBasis],
    precompute_basis: &PrecomputeMapBasis,
    mut request: Option<&mut MapKernelContext<'_, '_>>,
) -> Result<Result<u64, CapacityDenial>, SignalError> {
    let minimum = combined_map_requirement(
        bases,
        basis,
        candidate_bases,
        candidate_basis,
        precompute_bases,
        precompute_basis,
        0,
        lease,
        candidate_lease,
    )
    .ok_or_else(overflow)?;
    let lease_bytes = lease.memory_limit();
    if minimum > lease_bytes {
        return Ok(Err(CapacityDenial {
            required: minimum,
            reserved: lease_bytes,
        }));
    }
    if fixed > available {
        return Ok(Err(CapacityDenial {
            required: fixed.max(available.saturating_add(1)),
            reserved: available,
        }));
    }
    let mut lower = 0_u64;
    let mut upper = (available - fixed) / copies;
    while lower < upper {
        super::super::super::work::checkpoint(request.as_deref_mut(), 1)?;
        let middle = lower + (upper - lower) / 2 + (upper - lower) % 2;
        let fits = PartitionVersionOverrides::evaluation_growth_ceiling(middle)
            .ok()
            .and_then(|growth| {
                middle.checked_mul(copies).and_then(|copy_bytes| {
                    growth
                        .bytes()
                        .checked_mul(width as u64)
                        .and_then(|versions| copy_bytes.checked_add(versions))
                })
            })
            .is_some_and(|required| {
                required <= available - fixed
                    && combined_map_requirement(
                        bases,
                        basis,
                        candidate_bases,
                        candidate_basis,
                        precompute_bases,
                        precompute_basis,
                        middle,
                        lease,
                        candidate_lease,
                    )
                    .is_some_and(|bytes| bytes <= lease_bytes)
            });
        if fits {
            lower = middle;
        } else {
            upper = middle - 1;
        }
    }
    if lower < declared_result_minimum {
        let preparation_required =
            PartitionVersionOverrides::evaluation_growth_ceiling(declared_result_minimum)
                .ok()
                .and_then(|growth| {
                    declared_result_minimum
                        .checked_mul(copies)
                        .and_then(|results| {
                            growth
                                .bytes()
                                .checked_mul(width as u64)
                                .and_then(|versions| results.checked_add(versions))
                        })
                        .and_then(|variable| fixed.checked_add(variable))
                })
                .unwrap_or(u64::MAX);
        let map_required = combined_map_requirement(
            bases,
            basis,
            candidate_bases,
            candidate_basis,
            precompute_bases,
            precompute_basis,
            declared_result_minimum,
            lease,
            candidate_lease,
        )
        .unwrap_or(u64::MAX);
        let (required, reserved) = if preparation_required > available {
            (preparation_required, available)
        } else {
            (map_required, lease_bytes)
        };
        return Ok(Err(CapacityDenial { required, reserved }));
    }
    Ok(Ok(lower))
}

fn combined_map_requirement(
    apply_bases: &[ApplyMemberBasis],
    apply_basis: &ApplyMemberBasis,
    candidate_bases: &[CandidateEpochBasis],
    candidate_basis: &CandidateEpochBasis,
    precompute_bases: &[PrecomputeMapBasis],
    precompute_basis: &PrecomputeMapBasis,
    grant: u64,
    request_lease: worth_execution::ExecutionRequest<'_, '_>,
    child_lease: Option<&ExecutionResourceLease<'_>>,
) -> Option<u64> {
    let apply =
        apply_capacity::map_memory_requirement(apply_bases, apply_basis, grant, child_lease)?;
    let candidates =
        candidate_map_memory_requirement(candidate_bases, candidate_basis, grant, child_lease)?;
    let precompute =
        checked_map_memory_requirement(precompute_bases, precompute_basis, grant, child_lease)?;
    // The enclosing Signal scan holds both declared memory allowances while
    // these two prepared tickets coexist. Its generic framework context is
    // checked again by the authoritative pre-callback prepare_run calls.
    let request_memory = request_lease.memory_limit();
    let held_request = request_memory / 8 + request_memory / 8;
    held_request
        .checked_add(apply)?
        .checked_add(candidates)?
        .checked_add(precompute)
}
