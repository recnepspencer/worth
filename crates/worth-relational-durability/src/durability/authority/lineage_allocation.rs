//! Checkpoint and replay use the canonical lineage allocation reader.
#[cfg(test)]
pub(crate) use worth_relational_lineage::internal_port::lineage::authority::committed_allocations::LineageAllocationRefusal;
pub(crate) use worth_relational_lineage::internal_port::lineage::authority::committed_allocations::{
    allocated_lineage, commit_allocations, entity_allocation_bindings, explicit_entity_bindings,
    maximum_allocated_lineage, CommitAllocationView,
};
