//! Resolve one writer's normalized suffix against the actual applied prefix bindings.
use super::*;
use crate::contracts::transactions::data::outcomes::created_entity_bindings::CommitCreatedEntityBindings;
use crate::contracts::transactions::data::primitives::EntityReference;
impl OwnedNormalizedPlan {
    pub fn into_coupled_suffix(
        mut self,
        prefix: usize,
        bindings: &CommitCreatedEntityBindings,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<Self, TransactionCommitError> {
        if prefix > self.plan.merged_intents.len() {
            return Err(
                crate::contracts::transactions::data::outcomes::transaction_commit_error::origin_denial(
                    "coupled suffix starts beyond its normalized source",
                ),
            );
        }
        request
            .run_native_with_checkpoint(|context| {
                use crate::contracts::execution::request_stage::checkpoint;
                let copy = self
                    .plan
                    .merged_intents
                    .len()
                    .checked_sub(prefix)
                    .and_then(|count| count.checked_mul(std::mem::size_of::<MutationIntent>()))
                    .and_then(|bytes| u64::try_from(bytes).ok())
                    .ok_or_else(copy_overflow)?;
                checkpoint(context, copy).map_err(stop)?;
                for intent in &self.plan.merged_intents[..prefix] {
                    let units = intent
                        .copy_storage_bytes()
                        .and_then(|bytes| u64::try_from(bytes).ok())
                        .and_then(|bytes| bytes.checked_add(1))
                        .ok_or_else(copy_overflow)?;
                    checkpoint(context, units).map_err(stop)?;
                }
                self.plan.merged_intents.drain(..prefix);
                for intent in &mut self.plan.merged_intents {
                    checkpoint(context, 1).map_err(stop)?;
                    intent.visit_entity_references_mut(|reference| {
                        if let EntityReference::Created(created) = reference {
                            checkpoint(
                                context,
                                bindings
                                    .resolve_work_bound(created)
                                    .ok_or_else(copy_overflow)?,
                            )
                            .map_err(stop)?;
                            if let Some(id) = bindings.resolve(created) {
                                *reference = EntityReference::Existing(id);
                            }
                        }
                        Ok::<_, TransactionCommitError>(())
                    })?;
                }
                Ok::<_, TransactionCommitError>(())
            })
            .map_err(request_denial)?
            .0?;
        // Drain retains the source vector capacity; its original reservation stays
        // with this owner through sorting, candidate retention and final completion.
        Ok(self)
    }
}
fn stop(
    cause: impl Into<crate::contracts::execution::denial_cause::RelationalExecutionDenialCause>,
) -> TransactionCommitError {
    TransactionCommitError::execution(crate::contracts::transactions::data::outcomes::transaction_commit_error::CommitExecutionDenial {
        kind: crate::contracts::transactions::data::outcomes::transaction_commit_error::CommitExecutionDenialKind::Cause(cause.into()),
        partition_identity: None,
    })
}
