//! Each admitted participant keeps its local Create coordinates in the combined transaction.
use super::*;
use crate::execution::request_stage::checkpoint;
use crate::{
    symbols::data::{ClientKey, StringInterner},
    transactions::data::{CommitCreatedEntityBindings, EntityReference, WorkerIntentBatch},
};
use std::fmt::Write;

/// The scoped coordinates one part chose, by original text. A successor round
/// of the same route reuses exactly these, so its keys name the chain's records.
pub(super) type LocalKeys = std::collections::BTreeMap<String, String>;

pub(super) fn scope(
    execution: &ExecutionRequest<'_, '_>,
    part: usize,
    prior: &CommitCreatedEntityBindings,
    symbols: &StringInterner,
    batches: &mut [WorkerIntentBatch],
    roles: &mut [RequiredOutputRoleTarget],
) -> Result<LocalKeys, TransactionCommitError> {
    let mut chosen = LocalKeys::new();
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
                    checkpoint(context, capacity as u64).map_err(custody::execution_stop)?;
                    let mut name = String::new();
                    name.try_reserve_exact(capacity).map_err(|_| {
                        custody::execution_stop(
                            crate::execution::RelationalExecutionDenialCause::AllocationFailed,
                        )
                    })?;
                    write!(name, "WORTH.coupled.{part}.{salt}.{}:{text}", text.len())
                        .expect("reserved String framing");
                    if !prior.contains_client_key_text(&name, |work| {
                        checkpoint(context, work).map_err(custody::execution_stop)
                    })? {
                        break name;
                    }
                    salt = salt.checked_add(1).ok_or_else(overflow)?;
                };
                // The ordinary transaction staging owner admits the rewritten batch
                // backing before it can enter the Native proposal.
                chosen.insert(text.to_owned(), name.clone());
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
            Ok::<_, TransactionCommitError>(())
        })
        .map_err(custody::execution_stop)?
        .0?;
    Ok(chosen)
}

/// Rewrite a successor round's keys to the coordinates its chain head chose.
/// `Err(())` when the successor names a key the head never did: a new record
/// or reference that a fold cannot lower in place.
pub(super) fn rescope(
    execution: &ExecutionRequest<'_, '_>,
    head: &LocalKeys,
    symbols: &StringInterner,
    batches: &mut [WorkerIntentBatch],
    roles: &mut [RequiredOutputRoleTarget],
) -> Result<Result<(), ()>, TransactionCommitError> {
    execution
        .run_native_with_checkpoint(|context| {
            let mut unknown = false;
            let mut rewrite = |key: &mut ClientKey| {
                let text = key.resolve_with_interner(symbols).ok_or_else(|| {
                    crate::mvcc::origin_denial(
                        "coupled local key has no actual Native symbol meaning",
                    )
                })?;
                checkpoint(context, text.len() as u64 + 1).map_err(custody::execution_stop)?;
                match head.get(text) {
                    Some(name) => *key = ClientKey::raw(name.clone()),
                    None => unknown = true,
                }
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
            Ok(if unknown { Err(()) } else { Ok(()) })
        })
        .map_err(custody::execution_stop)?
        .0
}
/// A successor round's attempt issues its own mutation partition; the chain's
/// records live where its head allocated them. Each rescoped key names one
/// chain record, so its creates and created references take the head's
/// partition for that key.
pub(super) fn repartition(
    execution: &ExecutionRequest<'_, '_>,
    head: &[crate::transactions::data::MutationIntent],
    symbols: &StringInterner,
    batches: &mut [WorkerIntentBatch],
    roles: &mut [RequiredOutputRoleTarget],
) -> Result<(), TransactionCommitError> {
    use crate::transactions::data::{CreateIntent, MutationIntent};
    execution
        .run_native_with_checkpoint(|context| {
            let mut partitions = std::collections::BTreeMap::new();
            for intent in head {
                if let MutationIntent::Create(CreateIntent::Entity(spec)) = intent {
                    let text = spec
                        .client_key
                        .resolve_with_interner(symbols)
                        .ok_or_else(|| {
                            crate::mvcc::origin_denial(
                                "coupled local key has no actual Native symbol meaning",
                            )
                        })?;
                    checkpoint(context, text.len() as u64 + 1).map_err(custody::execution_stop)?;
                    partitions.insert(text.to_owned(), spec.partition_id);
                }
            }
            let head_partition = |key: &ClientKey| {
                key.as_raw_str()
                    .and_then(|text| partitions.get(text))
                    .copied()
            };
            let mut repartition = |reference: &mut EntityReference| {
                if let EntityReference::Created(created) = reference
                    && let Some(partition) = head_partition(&created.client_key)
                {
                    created.partition_id = partition;
                }
                Ok::<_, TransactionCommitError>(())
            };
            for batch in batches {
                for intent in &mut batch.intents {
                    checkpoint(context, 1).map_err(custody::execution_stop)?;
                    if let MutationIntent::Create(CreateIntent::Entity(spec)) = intent
                        && let Some(partition) = head_partition(&spec.client_key)
                    {
                        spec.partition_id = partition;
                    }
                    intent.visit_entity_references_mut(&mut repartition)?;
                }
            }
            for role in roles {
                checkpoint(context, 1).map_err(custody::execution_stop)?;
                repartition(&mut role.target)?;
            }
            Ok::<_, TransactionCommitError>(())
        })
        .map_err(custody::execution_stop)?
        .0
}
fn overflow() -> TransactionCommitError {
    custody::execution_stop(crate::execution::RelationalExecutionDenialCause::ChargedBytesOverflow)
}
