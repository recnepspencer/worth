//! Retained Source identity owns no live cell, newer image or execution request.
use crate::history::data::BranchId;
use std::sync::Arc;

pub struct ManagedSourceCellAffinity {
    pub runtime: u64,
    pub branch: BranchId,
    pub generation: u64,
    pub _memory: Option<worth_execution::ExecutionMemoryReservation>,
    // Weak<Core> still owns its Arc allocation after Core value destruction.
    // Share the existing fork quote through the last retained affinity owner.
    pub _fork_memory: Option<Arc<worth_execution::ExecutionMemoryReservation>>,
}
