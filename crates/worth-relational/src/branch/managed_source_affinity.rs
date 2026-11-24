//! Retained Source identity owns no live cell, newer image or execution request.
use crate::history::data::BranchId;
use std::sync::Arc;

pub(crate) struct ManagedSourceCellAffinity {
    pub(crate) runtime: u64,
    pub(crate) branch: BranchId,
    pub(crate) generation: u64,
    pub(crate) _memory: Option<worth_execution::ExecutionMemoryReservation>,
    // Weak<Core> still owns its Arc allocation after Core value destruction.
    // Share the existing fork quote through the last retained affinity owner.
    pub(crate) _fork_memory: Option<Arc<worth_execution::ExecutionMemoryReservation>>,
}
