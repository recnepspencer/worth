//! Each admitted participant keeps its local Create coordinates in the combined transaction.
use super::*;
use crate::{
    symbols::data::{ClientKey, StringInterner},
    transactions::data::{CommitCreatedEntityBindings, EntityReference, WorkerIntentBatch},
};
use std::fmt::Write;

pub(super) fn scope(
    execution: &ExecutionRequest<'_, '_>,
    part: usize,
    prior: &CommitCreatedEntityBindings,
    symbols: &StringInterner,
    batches: &mut [WorkerIntentBatch],
    roles: &mut [RequiredOutputRoleTarget],
) -> Result<(), TransactionCommitError> {
    execution
        .run_native_with_checkpoint(|context| {
            let mut rewrite = |key: &mut ClientKey| {
                let text = key.resolve_with_interner(symbols).ok_or_else(|| {
                    crate::mvcc::origin_denial(
                        "coupled local key has no actual Native symbol meaning",
                    )
                })?;
                // The framing is injective in the part, salt and original text. Every
                // occurrence of one local key chooses the same first unused prefix
                // coordinate; no second correspondence map is needed.
                let capacity = text.len().checked_add(128).ok_or_else(overflow)?;
                let mut salt = 0u64;
                let name = loop {
                    context
                        .checkpoint(capacity as u64)
                        .map_err(custody::execution_stop)?;
                    let mut name = String::new();
                    name.try_reserve_exact(capacity).map_err(|_| {
                        custody::execution_stop(
                            crate::execution::RelationalExecutionDenialCause::AllocationFailed,
                        )
                    })?;
                    write!(name, "WORTH.coupled.{part}.{salt}.{}:{text}", text.len())
                        .expect("reserved String framing");
                    if !prior.contains_client_key_text(&name, |work| {
                        context.checkpoint(work).map_err(custody::execution_stop)
                    })? {
                        break name;
                    }
                    salt = salt.checked_add(1).ok_or_else(overflow)?;
                };
                // The ordinary transaction staging owner admits the rewritten batch
                // backing before it can enter the Native proposal.
                *key = ClientKey::raw(name);
                Ok::<_, TransactionCommitError>(())
            };
            for batch in batches {
                for intent in &mut batch.intents {
                    intent.visit_client_keys_mut(&mut rewrite)?;
                }
            }
            for role in roles {
                if let EntityReference::Created(created) = &mut role.target {
                    rewrite(&mut created.client_key)?;
                }
            }
            Ok(())
        })
        .map_err(custody::execution_stop)?
        .0
}
fn overflow() -> TransactionCommitError {
    custody::execution_stop(crate::execution::RelationalExecutionDenialCause::ChargedBytesOverflow)
}
