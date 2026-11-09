//! Normalize an incoming coupled suffix before indexing it, never its retained prefix.
use super::BranchBoundRelationalTransaction;
use crate::{
    authority::mutation::MutationApplyDenial,
    runtime::{PendingSymbolMeanings, RelationalPreparationRuntime},
    symbols::data::ClientKeySymbolPolicy,
    transactions::data::{MutationIntent, TransactionCommitError, WorkerIntentBatch},
};
use worth_execution::{ExecutionAllocationPolicy, ExecutionRequest};
impl BranchBoundRelationalTransaction {
    pub(crate) fn reserve_coupled_batch_keys(
        &mut self,
        execution: &ExecutionRequest<'_, '_>,
        symbols: &mut PendingSymbolMeanings,
        batches: &[WorkerIntentBatch],
        policy: ClientKeySymbolPolicy,
        allocation: ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<(), MutationApplyDenial> {
        if !policy.interns_requested_strings() {
            return Ok(());
        }
        let mut raw = std::collections::BTreeSet::new();
        execution
            .run_native_with_checkpoint(|context| {
                for intent in batches.iter().flat_map(|batch| &batch.intents) {
                    crate::execution::request_stage::checkpoint(
                        context,
                        intent.owned_allocation_capacity_bytes().saturating_add(1),
                    )
                    .map_err(request_stop)?;
                    intent
                        .collect_raw_client_keys(&mut raw, allocation)
                        .map_err(|cause| {
                            TransactionCommitError::conflict(
                                super::RelationalTransactionStagingDenial::from(cause)
                                    .into_conflict(),
                            )
                        })?;
                }
                for text in raw {
                    crate::execution::request_stage::checkpoint(
                        context,
                        text.len().saturating_add(1) as u64,
                    )
                    .map_err(request_stop)?;
                    let symbol = symbols.reserve(&text)?;
                    self.overlay.normalized_client_keys.insert(text, symbol);
                }
                Ok::<_, MutationApplyDenial>(())
            })
            .map_err(request_stop)?
            .0?;
        Ok(())
    }
    pub(crate) fn push_normalized_coupled_batches(
        &mut self,
        execution: &ExecutionRequest<'_, '_>,
        runtime: &RelationalPreparationRuntime,
        mut batches: Vec<WorkerIntentBatch>,
    ) -> Result<Vec<MutationIntent>, TransactionCommitError> {
        let allocation = execution.allocation_policy();
        let policy = runtime.config.identity.client_key_symbol_policy;
        let mut normalized = Ok(());
        runtime.services.symbols.normalize_client_keys(|symbols| {
            normalized = super::overlay_normalization::normalize_intents(
                batches.iter_mut().flat_map(|batch| &mut batch.intents),
                symbols,
                policy,
                allocation,
            );
            // The reservation owner already retained every incoming text and its
            // dictionary identity. This phase only rewrites those exact keys.
            Vec::new()
        });
        normalized.map_err(|cause| TransactionCommitError::conflict(cause.into_conflict()))?;
        let mut intents = Vec::new();
        execution
            .run_native_with_checkpoint(|context| {
                for batch in &batches {
                    for intent in &batch.intents {
                        crate::execution::request_stage::checkpoint(
                            context,
                            intent.owned_allocation_capacity_bytes().saturating_add(1),
                        )
                        .map_err(request_stop)?;
                        intents.try_reserve(1).map_err(|_| {
                            request_stop(
                                crate::execution::RelationalExecutionDenialCause::AllocationFailed,
                            )
                        })?;
                        intents.push(intent.clone());
                    }
                }
                Ok::<_, TransactionCommitError>(())
            })
            .map_err(request_stop)?
            .0?;
        for batch in batches {
            self.push_batch(batch, allocation)
                .map_err(|cause| TransactionCommitError::conflict(cause.into_conflict()))?;
        }
        Ok(intents)
    }
}

fn request_stop(
    cause: impl Into<crate::execution::RelationalExecutionDenialCause>,
) -> TransactionCommitError {
    TransactionCommitError::execution(crate::transactions::data::CommitExecutionDenial {
        kind: crate::transactions::data::CommitExecutionDenialKind::Cause(cause.into()),
        partition_identity: None,
    })
}
