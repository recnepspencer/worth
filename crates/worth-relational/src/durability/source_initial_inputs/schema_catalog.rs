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
        let memory = request
            .reserve_retained_memory(layout.size() as u64)
            .map_err(|cause| RelationalRecoveryStop::Request(cause.into()))?;
        Ok(Arc::new(Self {
            rows: Default::default(),
            _memory: memory,
        }))
    }
}
