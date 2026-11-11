//! Read-only traversal of native payload allocations at their owner granularity.
//!
//! Native nodes and payload carriers are identified by their real allocation.
//! Persistent aspect backing nested in a carrier arrives as that carrier's
//! checked declared upper bound, so backing shared by distinct carriers is
//! counted once per carrier. An unrepresentable bound refuses the walk.
use super::allocation_ledger::RelationalExcludedAllocationLane;
use super::sharing::{
    RelationalAuthoritativeAllocationKind, RelationalAuthoritativeAllocationLocator,
    RelationalAuthoritativeAllocationObservation, RelationalBranchSharingInspectionDenial,
};
use crate::identity::data::PartitionId;
use crate::storage::{
    overlay::PartitionState,
    substrate::{StorageAllocationExtent, StorageAllocationObservation, StorageAllocationVisitor},
};

/// Admits one owner observation's checked extent before it becomes evidence.
pub(super) fn admitted_bytes(
    allocation: StorageAllocationObservation,
) -> Result<u64, RelationalBranchSharingInspectionDenial> {
    match allocation.extent {
        StorageAllocationExtent::Checked => Ok(allocation.bytes),
        StorageAllocationExtent::Overflow => {
            Err(RelationalBranchSharingInspectionDenial::AuthoritativeByteBoundOverflow)
        }
    }
}

pub(super) fn authoritative_partition_allocations(
    runtime_instance_id: u64,
    partition: &PartitionState,
    seen: &mut std::collections::BTreeSet<u64>,
    work: Option<&mut super::retained_storage::RelationalRetainedStorageWork>,
) -> Result<
    Vec<RelationalAuthoritativeAllocationObservation>,
    RelationalBranchSharingInspectionDenial,
> {
    let mut walk = PartitionAllocationWalk {
        runtime_instance_id,
        partition_id: partition.partition_id,
        seen,
        allocations: Vec::new(),
        work,
        denial: None,
    };
    partition.visit_authoritative_allocations(false, &mut walk);
    match walk.denial {
        Some(denial) => Err(denial),
        None => Ok(walk.allocations),
    }
}

struct PartitionAllocationWalk<'a> {
    runtime_instance_id: u64,
    partition_id: PartitionId,
    seen: &'a mut std::collections::BTreeSet<u64>,
    allocations: Vec<RelationalAuthoritativeAllocationObservation>,
    work: Option<&'a mut super::retained_storage::RelationalRetainedStorageWork>,
    denial: Option<RelationalBranchSharingInspectionDenial>,
}

impl StorageAllocationVisitor for PartitionAllocationWalk<'_> {
    fn visit(&mut self, allocation: StorageAllocationObservation) -> bool {
        if let Some(work) = self.work.as_mut() {
            work.payload_allocation_attempts += 1;
        }
        let bytes = match admitted_bytes(allocation) {
            Ok(bytes) => bytes,
            Err(denial) => {
                self.denial = Some(denial);
                return false;
            }
        };
        if !self.seen.insert(allocation.id) {
            return false;
        }
        if let Some(work) = self.work.as_mut() {
            work.payload_new_allocations += 1;
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
                bytes,
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
