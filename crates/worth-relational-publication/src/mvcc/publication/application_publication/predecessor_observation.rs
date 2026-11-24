//! Member coverage of an admitted output predecessor.
use super::*;
use contracts::identity::data::record_ids::EntityId;
use worth_execution::ExecutionRequest;
use worth_relational_contracts::internal_port as contracts;
impl RelationalRequiredOutputPublication {
    /// Canonical member coverage of an already admitted complete predecessor.
    pub fn contains_output_entity(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        entity: EntityId,
    ) -> Result<bool, contracts::execution::denial_cause::RelationalExecutionDenialCause> {
        let result = execution
            .run_native_with_checkpoint(|context| {
                contracts::execution::request_stage::checkpoint(context, 1).map_err(
                    contracts::execution::denial_cause::RelationalExecutionDenialCause::from,
                )?;
                let work = u64::from(usize::BITS - self.origin.outputs.len().leading_zeros()) + 1;
                contracts::execution::request_stage::checkpoint(context, work).map_err(
                    contracts::execution::denial_cause::RelationalExecutionDenialCause::from,
                )?;
                Ok(self.origin.outputs.binary_search(&entity).is_ok())
            })
            .map_err(contracts::execution::denial_cause::RelationalExecutionDenialCause::from)?;
        result.0
    }
}
