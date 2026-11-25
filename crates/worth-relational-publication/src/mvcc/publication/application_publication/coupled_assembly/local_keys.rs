//! Each admitted participant keeps its local Create coordinates in the combined transaction.
use crate::mvcc::publication::application_publication::custody;
use contracts::execution::request_stage::checkpoint;
use contracts::lineage::data::CommitNamingPlan;
use contracts::mvcc::publication::application_publication::output_meaning::RequiredOutputRoleTarget;
use contracts::symbols::data::client_key::ClientKey;
use contracts::symbols::data::string_interner::StringInterner;
use contracts::transactions::data::outcomes::created_entity_bindings::CommitCreatedEntityBindings;
use contracts::transactions::data::outcomes::transaction_commit_error::TransactionCommitError;
use contracts::transactions::data::primitives::EntityReference;
use contracts::transactions::data::primitives::WorkerIntentBatch;
use std::fmt::Write;
use worth_execution::ExecutionRequest;
use worth_relational_contracts::internal_port as contracts;

/// The scoped coordinates one part chose, by original text. A successor round
/// of the same route reuses exactly these, so its keys name the chain's records.
pub type LocalKeys = std::collections::BTreeMap<String, String>;

pub fn scope(
    execution: &ExecutionRequest<'_, '_>,
    part: usize,
    prior: &CommitCreatedEntityBindings,
    symbols: &StringInterner,
    batches: &mut [WorkerIntentBatch],
    roles: &mut [RequiredOutputRoleTarget],
    naming: Option<&mut CommitNamingPlan>,
) -> Result<LocalKeys, TransactionCommitError> {
    let mut chosen = LocalKeys::new();
    execution
        .run_native_with_checkpoint(|context| {
            let mut rewrite = |key: &mut ClientKey| {
                let text = key.resolve_with_interner(symbols).ok_or_else(|| {
                    contracts::transactions::data::outcomes::transaction_commit_error::origin_denial(
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
                            contracts::execution::denial_cause::RelationalExecutionDenialCause::AllocationFailed,
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
            if let Some(plan) = naming {
                super::part_naming::visit_client_keys_mut(plan, &mut rewrite)?;
            }
            Ok::<_, TransactionCommitError>(())
        })
        .map_err(custody::execution_stop)?
        .0?;
    Ok(chosen)
}

/// Rewrite a successor round's keys to its chain's coordinates. A key no
/// round of the chain used gets a new coordinate in the head's framing
/// (`WORTH.coupled.{head}.{salt}.{len}:{text}`), salted against every key of
/// the candidate and the chain, and joins the chain's keys.
pub fn rescope(
    execution: &ExecutionRequest<'_, '_>,
    keys: &mut LocalKeys,
    head: usize,
    prior: &CommitCreatedEntityBindings,
    ledger: &super::succession::ledger::ChainCreateLedger,
    symbols: &StringInterner,
    batches: &mut [WorkerIntentBatch],
    roles: &mut [RequiredOutputRoleTarget],
    naming: Option<&mut CommitNamingPlan>,
) -> Result<(), TransactionCommitError> {
    execution
        .run_native_with_checkpoint(|context| {
            let mut rewrite = |key: &mut ClientKey| {
                let text = key.resolve_with_interner(symbols).ok_or_else(|| {
                    contracts::transactions::data::outcomes::transaction_commit_error::origin_denial(
                        "coupled local key has no actual Native symbol meaning",
                    )
                })?;
                checkpoint(context, text.len() as u64 + 1).map_err(custody::execution_stop)?;
                if let Some(name) = keys.get(text) {
                    *key = ClientKey::raw(name.clone());
                    return Ok(());
                }
                let capacity = text.len().checked_add(128).ok_or_else(overflow)?;
                let mut salt = 0u64;
                let name = loop {
                    checkpoint(context, capacity as u64).map_err(custody::execution_stop)?;
                    let mut name = String::new();
                    name.try_reserve_exact(capacity).map_err(|_| {
                        custody::execution_stop(
                            contracts::execution::denial_cause::RelationalExecutionDenialCause::AllocationFailed,
                        )
                    })?;
                    write!(name, "WORTH.coupled.{head}.{salt}.{}:{text}", text.len())
                        .expect("reserved String framing");
                    let taken = ledger.knows_text(&name)
                        || keys.values().any(|chosen| *chosen == name)
                        || prior.contains_client_key_text(&name, |work| {
                            checkpoint(context, work).map_err(custody::execution_stop)
                        })?;
                    if !taken {
                        break name;
                    }
                    salt = salt.checked_add(1).ok_or_else(overflow)?;
                };
                keys.insert(text.to_owned(), name.clone());
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
            if let Some(plan) = naming {
                super::part_naming::visit_client_keys_mut(plan, &mut rewrite)?;
            }
            Ok::<_, TransactionCommitError>(())
        })
        .map_err(custody::execution_stop)?
        .0
}

mod repartition;
pub use repartition::repartition;

pub(super) fn overflow() -> TransactionCommitError {
    custody::execution_stop(
        contracts::execution::denial_cause::RelationalExecutionDenialCause::ChargedBytesOverflow,
    )
}
