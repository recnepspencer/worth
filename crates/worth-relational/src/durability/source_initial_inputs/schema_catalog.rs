//! One request-funded immutable schema graph follows exact source readmission.
use crate::branch::RelationalBranchRootSchemaAuthority;
use crate::durability::data::RelationalRecoveryStop;
use std::sync::Arc;
use worth_execution::{ExecutionMemoryReservation, ExecutionOrderedMap, ExecutionRequest};

#[derive(Debug)]
pub(crate) struct ReadmittedSourceSchemaCatalog {
    pub(crate) rows: ExecutionOrderedMap<[u8; 32], Arc<RelationalBranchRootSchemaAuthority>>,
    _memory: Option<ExecutionMemoryReservation>,
}
impl ReadmittedSourceSchemaCatalog {
    pub(crate) fn prepare(
        request: &ExecutionRequest<'_, '_>,
        directory_copies: usize,
    ) -> Result<Arc<Self>, RelationalRecoveryStop> {
        let layout = std::alloc::Layout::new::<[usize; 2]>()
            .extend(std::alloc::Layout::new::<Self>())
            .map_err(|_| {
                RelationalRecoveryStop::Request(
                    crate::execution::RelationalExecutionDenialCause::ChargedBytesOverflow,
                )
            })?
            .0
            .pad_to_align();
        let directory = std::alloc::Layout::new::<[usize; 2]>()
            .extend(std::alloc::Layout::new::<
                crate::durability::data::RelationalCheckpointDirectory,
            >())
            .map_err(|_| {
                crate::durability::checkpoints::region_directory::corrupt(
                    "directory binding layout overflow",
                )
            })?
            .0
            .pad_to_align()
            .size();
        let bytes = directory
            .checked_mul(directory_copies)
            .and_then(|copies| copies.checked_add(layout.size()))
            .ok_or_else(|| {
                crate::durability::checkpoints::region_directory::corrupt(
                    "directory binding bill overflow",
                )
            })?;
        // These attempt-local directory bindings retain this catalog, so its
        // original grant follows every actual fixed-size binding allocation.
        let memory = request
            .reserve_retained_memory(bytes as u64)
            .map_err(|cause| RelationalRecoveryStop::Request(cause.into()))?;
        Ok(Arc::new(Self {
            rows: Default::default(),
            _memory: memory,
        }))
    }
}
