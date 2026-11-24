pub mod access;
pub mod current_access;
pub use current_access::CurrentPartitionAccess;
pub mod partition;
pub mod partition_allocation_delta;
pub mod partition_allocation_walk;
pub mod partition_content;
pub mod partition_symbols;
pub mod result_partition_ids;
pub use result_partition_ids::partition_ids_in_result_memory;
pub mod slot_resolution;
pub mod working_state;
pub use slot_resolution::{
    current_entity_partition_source_for_slot, current_relation_partition_source_for_slot,
    PartitionSource,
};

pub use access::{
    collect_partition_ids, OverlayStateView, PartitionAccess, PartitionIdMemory, PartitionIdsDenial,
};
pub use partition::{
    summarize_entity_chunk_plan, EntityWorkingSetLayout, PartitionCloneMode,
    PartitionMutationJournal, PartitionState, RelationalPartitionAllocationInventory,
    SnapshotPartitionPins, TouchedSlots,
};
pub use partition_allocation_walk::ReclaimableAllocations;
pub use partition_content::{
    PartitionContentCommitment, PartitionContentDigestError, PartitionHistoryFrontiers,
};
pub use working_state::WorkingState;

pub mod mirror_merge;
pub mod published_delta;
pub mod record_observation;

#[doc(hidden)]
pub mod small_relation_partition;
