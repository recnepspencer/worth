//! Prospective checked precompute map while both apply tickets remain held.
//! Its generic memory formula belongs beside the actual GraphWorkItem map.

use std::mem::size_of;

use worth_execution::{ExecutionMap, ExecutionResourceLease};
use worth_foundational::PartitionIdentity;

use crate::data::aspect::MAX_ASPECTS;
use crate::data::error::SignalError;
use crate::data::proof::invalidation::progression::GraphProposalKey;
use crate::logic::prepared::PreparedEvaluation;

use super::map_declaration::GraphWorkItem;

pub(in crate::logic::planner::precompute) struct PrecomputeMapBasis {
    capture_bytes: u64,
    read_count: usize,
}

impl PrecomputeMapBasis {
    pub(in crate::logic::planner::precompute) fn new(
        capture_bytes: u64,
        read_count: usize,
    ) -> Self {
        Self {
            capture_bytes,
            read_count,
        }
    }
}

pub(in crate::logic::planner::precompute) fn checked_map_memory_requirement(
    prior: &[PrecomputeMapBasis],
    candidate: &PrecomputeMapBasis,
    grant: u64,
    lease: &ExecutionResourceLease<'_>,
) -> Option<u64> {
    let mut count = 0_usize;
    let mut scratch = 0_u64;
    let mut result = 0_u64;
    let mut access = 0_u64;
    for basis in prior.iter().chain(std::iter::once(candidate)) {
        count = count.checked_add(1)?;
        scratch = scratch.checked_add(grant.checked_add(basis.capture_bytes.checked_mul(2)?)?)?;
        result = result.checked_add(grant.checked_add(basis.capture_bytes)?)?;
        let keys = basis.read_count.checked_add(MAX_ASPECTS + 7)?;
        let key_bytes = keys.checked_mul(size_of::<GraphProposalKey>())?;
        let member_bytes = size_of::<Vec<GraphProposalKey>>()
            .checked_add(size_of::<(PartitionIdentity, Vec<GraphProposalKey>)>())?;
        access = access.checked_add(u64::try_from(key_bytes.checked_add(member_bytes)?).ok()?)?;
    }
    ExecutionMap::<GraphWorkItem, GraphProposalKey>::declared_memory_requirement_for_lease::<
        PreparedEvaluation,
        SignalError,
    >(lease, count, 0, scratch, result, access)
}
