//! Member coverage of an admitted output predecessor.
use super::*;
use crate::identity::data::EntityId;
use worth_execution::ExecutionRequest;
impl RelationalRequiredOutputPublication {
    /// Canonical member coverage of an already admitted complete predecessor.
    pub fn contains_output_entity(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        entity: EntityId,
    ) -> Result<bool, crate::execution::RelationalExecutionDenialCause> {
        let result = execution
            .run_native_with_checkpoint(|context| {
                crate::execution::request_stage::checkpoint(context, 1)
                    .map_err(crate::execution::RelationalExecutionDenialCause::from)?;
                let work = u64::from(usize::BITS - self.origin.outputs.len().leading_zeros()) + 1;
                crate::execution::request_stage::checkpoint(context, work)
                    .map_err(crate::execution::RelationalExecutionDenialCause::from)?;
                Ok(self.origin.outputs.binary_search(&entity).is_ok())
            })
            .map_err(crate::execution::RelationalExecutionDenialCause::from)?;
        result.0
    }
}
