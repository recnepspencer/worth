//! Original body and original-request funding share exactly one lifetime.
use super::super::WorthQueryApplicationProjectionNode;
use std::sync::Arc;
use worth_query_declaration::facade::application_query::ApplicationQueryResultSlotKey;

pub(in crate::domain_computation::primary_graph) struct OriginalProjection {
    pub(super) rows: Vec<WorthQueryApplicationProjectionNode>,
    // Every declared field/relation decision, including withheld slots.
    // The original compiled binding interprets these exact shared slot keys.
    pub(super) disclosure:
        worth_execution::ExecutionArray<(Arc<ApplicationQueryResultSlotKey>, bool)>,
    // Successful construction is only for complete non-continuation trees.
    // Row source-dependency completeness is separate provenance, not payload.
    pub(super) _memory: Option<worth_execution::ExecutionMemoryReservation>,
}

impl OriginalProjection {
    pub(super) fn new(
        rows: Vec<WorthQueryApplicationProjectionNode>,
        disclosure: worth_execution::ExecutionArray<(Arc<ApplicationQueryResultSlotKey>, bool)>,
        memory: Option<worth_execution::ExecutionMemoryReservation>,
    ) -> Self {
        Self {
            rows,
            disclosure,
            _memory: memory,
        }
    }

    pub(super) fn allocation_bytes() -> u64 {
        std::alloc::Layout::new::<[std::sync::atomic::AtomicUsize; 2]>()
            .extend(std::alloc::Layout::new::<Self>())
            .expect("fixed original projection layout")
            .0
            .pad_to_align()
            .size() as u64
    }
}
