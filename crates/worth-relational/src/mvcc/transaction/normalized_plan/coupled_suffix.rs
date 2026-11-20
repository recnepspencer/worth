//! Resolve one writer's normalized suffix against the actual applied prefix bindings.
use super::*;
use crate::transactions::data::{
    CommitCreatedEntityBindings, CreateIntent, CreatedEntityRef, EntityMutationIntent,
    EntityReference, UpdateEntityFieldsIntent,
};
impl OwnedNormalizedPlan {
    /// A successor round re-creates its chain's records under the same keys.
    /// Its applied plan writes them in place on the chain's allocations, so no
    /// second allocation exists. The transaction keeps the Create itself.
    pub(crate) fn lower_coupled_recreates(
        mut self,
        bindings: &CommitCreatedEntityBindings,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<Self, TransactionCommitError> {
        request
            .run_native_with_checkpoint(|context| {
                use crate::execution::request_stage::checkpoint;
                for intent in &mut self.plan.merged_intents {
                    checkpoint(context, 1).map_err(stop)?;
                    let MutationIntent::Create(CreateIntent::Entity(spec)) = intent else {
                        continue;
                    };
                    let created = CreatedEntityRef {
                        partition_id: spec.partition_id,
                        kind_id: spec.kind_id,
                        client_key: spec.client_key.clone(),
                    };
                    checkpoint(
                        context,
                        bindings
                            .resolve_work_bound(&created)
                            .ok_or_else(copy_overflow)?,
                    )
                    .map_err(stop)?;
                    if let Some(entity_id) = bindings.resolve(&created) {
                        *intent = MutationIntent::Entity(EntityMutationIntent::UpdateFields(
                            UpdateEntityFieldsIntent {
                                entity_id,
                                fields: std::mem::take(&mut spec.fields),
                            },
                        ));
                    }
                }
                Ok::<_, TransactionCommitError>(())
            })
            .map_err(request_denial)?
            .0?;
        Ok(self)
    }

    pub(crate) fn into_coupled_suffix(
        mut self,
        prefix: usize,
        bindings: &CommitCreatedEntityBindings,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<Self, TransactionCommitError> {
        if prefix > self.plan.merged_intents.len() {
            return Err(crate::mvcc::origin_denial(
                "coupled suffix starts beyond its normalized source",
            ));
        }
        request
            .run_native_with_checkpoint(|context| {
                use crate::execution::request_stage::checkpoint;
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
    cause: impl Into<crate::execution::RelationalExecutionDenialCause>,
) -> TransactionCommitError {
    TransactionCommitError::execution(crate::transactions::data::CommitExecutionDenial {
        kind: crate::transactions::data::CommitExecutionDenialKind::Cause(cause.into()),
        partition_identity: None,
    })
}
