//! Read-only traversal of native payload allocations at their real sharing granularity.
use super::allocation_ledger::RelationalExcludedAllocationLane;
use super::sharing::{
    RelationalAuthoritativeAllocationKind, RelationalAuthoritativeAllocationLocator,
    RelationalAuthoritativeAllocationObservation,
};
use crate::identity::data::PartitionId;
use crate::storage::{
    overlay::PartitionState,
    substrate::{StorageAllocationObservation, StorageAllocationVisitor},
};

pub(super) fn authoritative_partition_allocations(
    runtime_instance_id: u64,
    partition: &PartitionState,
    seen: &mut std::collections::BTreeSet<u64>,
) -> Vec<RelationalAuthoritativeAllocationObservation> {
    let mut walk = PartitionAllocationWalk {
        runtime_instance_id,
        partition_id: partition.partition_id,
        seen,
        allocations: Vec::new(),
    };
    partition.visit_authoritative_allocations(false, &mut walk);
    walk.allocations
}

struct PartitionAllocationWalk<'a> {
    runtime_instance_id: u64,
    partition_id: PartitionId,
    seen: &'a mut std::collections::BTreeSet<u64>,
    allocations: Vec<RelationalAuthoritativeAllocationObservation>,
}

impl StorageAllocationVisitor for PartitionAllocationWalk<'_> {
    fn visit(&mut self, allocation: StorageAllocationObservation) -> bool {
        if !self.seen.insert(allocation.id) {
            return false;
        }
        self.allocations
            .push(RelationalAuthoritativeAllocationObservation::new(
                RelationalAuthoritativeAllocationLocator::new(
                    self.runtime_instance_id,
                    RelationalAuthoritativeAllocationKind::PartitionPayload,
                    allocation.id,
                    allocation.id,
                    Some(self.partition_id),
                ),
                allocation.bytes,
            ));
        true
    }
}

pub(super) fn excluded_partition_allocations(
    partition: &PartitionState,
    root: &crate::branch::RelationalBranchRoot,
    seen: &mut std::collections::BTreeSet<u64>,
) -> Vec<(
    RelationalExcludedAllocationLane,
    StorageAllocationObservation,
)> {
    let mut walk = ExcludedAllocationWalk {
        lane: RelationalExcludedAllocationLane::Diagnostics,
        seen,
        allocations: Vec::new(),
    };
    partition.visit_diagnostic_allocations(&mut walk);
    walk.lane = RelationalExcludedAllocationLane::RetentionMetadata;
    partition.visit_retention_allocations(&mut walk);
    walk.lane = RelationalExcludedAllocationLane::OptionalCache;
    partition.visit_cache_allocations(&mut walk);
    root.visit_content_cache_allocations(partition.partition_id, &mut walk);
    walk.allocations
}

struct ExcludedAllocationWalk<'a> {
    lane: RelationalExcludedAllocationLane,
    seen: &'a mut std::collections::BTreeSet<u64>,
    allocations: Vec<(
        RelationalExcludedAllocationLane,
        StorageAllocationObservation,
    )>,
}
impl StorageAllocationVisitor for ExcludedAllocationWalk<'_> {
    fn visit(&mut self, allocation: StorageAllocationObservation) -> bool {
        if !self.seen.insert(allocation.id) {
            return false;
        }
        self.allocations.push((self.lane, allocation));
        true
    }
}
