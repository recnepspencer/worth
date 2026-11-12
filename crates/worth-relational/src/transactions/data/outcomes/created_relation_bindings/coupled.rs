//! Carry original relation inputs across sequential prefix-reference lowering.
use super::CommitCreatedRelationBindings;
use crate::symbols::data::SymbolTextLookup;
use crate::transactions::data::{
    CommitCreatedEntityBindings, CreateIntent, CreatedRelationRef, EntityReference, MutationIntent,
    TransactionCommitError,
};
use std::collections::BTreeMap;
use worth_execution::ExecutionRequest;

pub(crate) struct CoupledOriginalRelationCreates {
    originals: Vec<CreatedRelationRef>,
}

impl CoupledOriginalRelationCreates {
    pub(crate) fn capture(
        request: &ExecutionRequest<'_, '_>,
        intents: &[MutationIntent],
    ) -> Result<Self, TransactionCommitError> {
        let mut count = 0usize;
        visit(intents, |_| {
            count = count.checked_add(1).ok_or_else(overflow)?;
            Ok(())
        })?;
        // Like MutationWorkspace::into_created_bindings, live result
        // correspondence is an ordinary owned copy, outside Complete folding.
        // This roster adds no retained-memory admission contract to publishing.
        let mut originals = Vec::with_capacity(count);
        request
            .run_native_with_checkpoint(|context| {
                visit(intents, |original| {
                    crate::execution::request_stage::checkpoint(
                        context,
                        original.copy_bytes()? as u64,
                    )
                    .map_err(stop)?;
                    originals.push(original.to_owned());
                    Ok(())
                })
            })
            .map_err(stop)?
            .0?;
        Ok(Self { originals })
    }

    pub(crate) fn complete(
        self,
        request: &ExecutionRequest<'_, '_>,
        generated: CommitCreatedRelationBindings,
        prefix: &CommitCreatedEntityBindings,
        symbols: &dyn SymbolTextLookup,
    ) -> Result<CommitCreatedRelationBindings, TransactionCommitError> {
        request
            .run_native_with_checkpoint(|context| {
                complete_originals(self.originals, generated, prefix, symbols, |work| {
                    crate::execution::request_stage::checkpoint(context, work).map_err(stop)
                })
            })
            .map_err(stop)?
            .0
    }
}

fn complete_originals(
    originals: Vec<CreatedRelationRef>,
    mut generated: CommitCreatedRelationBindings,
    prefix: &CommitCreatedEntityBindings,
    symbols: &dyn SymbolTextLookup,
    mut charge: impl FnMut(u64) -> Result<(), TransactionCommitError>,
) -> Result<CommitCreatedRelationBindings, TransactionCommitError> {
    let mut original_map = BTreeMap::new();
    for original in originals {
        charge(1)?;
        let mut lowered = original.clone();
        for endpoint in [&mut lowered.source, &mut lowered.target] {
            if let EntityReference::Created(created) = endpoint {
                charge(prefix.resolve_work_bound(created).ok_or_else(overflow)?)?;
                if let Some(assigned) = prefix.resolve(created) {
                    *endpoint = EntityReference::Existing(assigned);
                }
            }
        }
        charge(
            generated
                .resolve_work_bound(&lowered)
                .ok_or_else(overflow)?,
        )?;
        let assigned = generated
            .by_exact_reference
            .remove(&lowered)
            .ok_or_else(|| {
                disagreement("the coupled apply omitted an original relation creation")
            })?;
        if original_map.insert(original, assigned).is_some() {
            return Err(disagreement(
                "the coupled original repeats a relation creation",
            ));
        }
    }
    if !generated.by_exact_reference.is_empty() {
        return Err(disagreement(
            "the coupled apply added an unassociated relation creation",
        ));
    }
    Ok(CommitCreatedRelationBindings::from_owner_map(
        original_map,
        symbols,
    ))
}

struct OriginalRelation<'a> {
    partition: crate::identity::data::PartitionId,
    kind: crate::identity::data::KindId,
    key: &'a crate::symbols::data::ClientKey,
    source: &'a EntityReference,
    target: &'a EntityReference,
}
impl OriginalRelation<'_> {
    fn copy_bytes(&self) -> Result<usize, TransactionCommitError> {
        let endpoint_bytes = |endpoint: &EntityReference| match endpoint {
            EntityReference::Existing(_) => 0,
            EntityReference::Created(created) => {
                created.client_key.owned_allocation_capacity_bytes()
            }
        };
        (std::mem::size_of::<CreatedRelationRef>() as u64)
            .checked_add(self.key.owned_allocation_capacity_bytes())
            .and_then(|bytes| bytes.checked_add(endpoint_bytes(self.source)))
            .and_then(|bytes| bytes.checked_add(endpoint_bytes(self.target)))
            .and_then(|bytes| usize::try_from(bytes).ok())
            .ok_or_else(overflow)
    }
    fn to_owned(&self) -> CreatedRelationRef {
        CreatedRelationRef {
            partition_id: self.partition,
            kind_id: self.kind,
            client_key: self.key.clone(),
            source: self.source.clone(),
            target: self.target.clone(),
        }
    }
}

fn visit(
    intents: &[MutationIntent],
    mut accept: impl FnMut(OriginalRelation<'_>) -> Result<(), TransactionCommitError>,
) -> Result<(), TransactionCommitError> {
    for intent in intents {
        match intent {
            MutationIntent::Create(create) => match create {
                CreateIntent::Relation(value) => accept(OriginalRelation {
                    partition: value.partition_id,
                    kind: value.kind_id,
                    key: &value.client_key,
                    source: &value.source,
                    target: &value.target,
                })?,
                CreateIntent::RelationAspects(value) => accept(OriginalRelation {
                    partition: value.partition_id,
                    kind: value.kind_id,
                    key: &value.client_key,
                    source: &value.source,
                    target: &value.target,
                })?,
                CreateIntent::BulkRelations(value) => {
                    if value.client_keys.len() != value.endpoints.len() {
                        return Err(disagreement(
                            "the original coupled relation batch has unequal keys and endpoints",
                        ));
                    }
                    for (key, (source, target)) in value.client_keys.iter().zip(&value.endpoints) {
                        accept(OriginalRelation {
                            partition: value.partition_id,
                            kind: value.kind_id,
                            key,
                            source,
                            target,
                        })?;
                    }
                }
                CreateIntent::Entity(_)
                | CreateIntent::EntityAspects(_)
                | CreateIntent::BulkEntities(_) => {}
            },
            MutationIntent::Entity(_)
            | MutationIntent::Relation(_)
            | MutationIntent::Materialization(_) => {}
        }
    }
    Ok(())
}
fn disagreement(detail: &str) -> TransactionCommitError {
    TransactionCommitError::conflict(crate::transactions::data::CommitConflict::new(
        crate::transactions::data::ConflictClass::KindSchemaMismatch {
            detail: detail.into(),
        },
    ))
}
fn overflow() -> TransactionCommitError {
    crate::mvcc::plan_copy_admission::copy_overflow()
}
fn stop(
    cause: impl Into<crate::execution::RelationalExecutionDenialCause>,
) -> TransactionCommitError {
    TransactionCommitError::execution(crate::transactions::data::CommitExecutionDenial {
        kind: crate::transactions::data::CommitExecutionDenialKind::Cause(cause.into()),
        partition_identity: None,
    })
}

#[cfg(test)]
mod tests;
