//! Rejudge every unchanged source row without inventing a write or event.
use worth_relational_contracts::internal_port::transactions::data::outcomes::transaction_commit_error::TransactionCommitErrorOwnerPort as _;
use crate::branch::{SelectedRelationalBranchRead, SelectedRelationalBranchState};
use crate::mvcc::BranchBoundRelationalTransaction;
use crate::runtime::RelationalPreparationRuntime;
use crate::transactions::data::*;
use crate::validation::engine::state_view::InvariantStateView;
use std::sync::Arc;
use worth_execution::{
    ExecutionArray, ExecutionOrderedMap, ExecutionRequest, ExecutionRequestArrayBuilder,
};

pub(crate) mod captured_records;
pub(crate) use captured_records::RelationalHomeTargetCoverage;
use captured_records::{array, ordered, overflow, staging};

impl BranchBoundRelationalTransaction {
    pub(crate) fn prepare_home_target_coverage(
        &mut self,
        execution: &ExecutionRequest<'_, '_>,
        runtime: &RelationalPreparationRuntime,
    ) -> Result<(), TransactionCommitError> {
        if self.home_target.is_none() || self.home_target_coverage.is_some() {
            return Ok(());
        }
        let selected = SelectedRelationalBranchState::from_admitted_basis(&self.basis);
        let records = crate::storage::residency::probe_head(
            execution,
            &selected.with_symbol_admission(&runtime.services.symbols),
            |partitions| {
                capture_records(
                    execution,
                    &crate::branch::SelectedRelationalBranchRead::in_round(&selected, partitions),
                )
            },
        )
        .map_err(|cause| {
            crate::authority::commit::preparation::storage_read_denial(&self.basis, cause)
        })??;
        let mut authored = ExecutionOrderedMap::default();
        for batch in self.batches() {
            for intent in &batch.intents {
                execution.consult()?;
                if let Some(target) = intent.existing_record_target() {
                    authored
                        .insert_cloned_native_for_request(&target, &(), *execution)
                        .map_err(ordered)?;
                }
            }
        }
        let count = records
            .iter()
            .filter(|record| !authored.contains_key(&target(record)))
            .count();
        let bytes = count
            .checked_mul(size_of::<MutationIntent>())
            .and_then(|bytes| bytes.checked_add("home-target-rejudgment".len()))
            .and_then(|bytes| {
                bytes
                    .checked_add(size_of::<RelationalHomeTargetCoverage>() + 2 * size_of::<usize>())
            })
            .ok_or_else(overflow)?;
        let memory = execution.reserve_retained_memory(bytes as u64)?;
        let mut batch = WorkerIntentBatch::new("home-target-rejudgment");
        batch.intents = Vec::with_capacity(count);
        for record in records.iter() {
            execution.consult()?;
            if authored.contains_key(&target(record)) {
                continue;
            }
            batch.intents.push(match record {
                RecordRef::Entity(entity_id) => MutationIntent::Entity(
                    EntityMutationIntent::Revalidate(RevalidateEntityIntent {
                        entity_id: *entity_id,
                    }),
                ),
                RecordRef::Relation(relation_id) => MutationIntent::Relation(
                    RelationMutationIntent::Revalidate(RevalidateRelationIntent {
                        relation_id: *relation_id,
                    }),
                ),
            });
        }
        let authored_batches = self.batches().len();
        let rejudgment_body = super::home_program_transition_body_digest(
            execution,
            &crate::mvcc::RelationalTransactionIntent::ordinary(),
            std::slice::from_ref(&batch),
        )
        .map_err(super::body_validation::body_denial)?;
        self.push_batch(batch, execution.allocation_policy())
            .map_err(staging)?;
        self.home_target_coverage = Some(Arc::new(RelationalHomeTargetCoverage {
            authored_batches,
            rejudgment_body,
            records,
            _batch_memory: memory,
        }));
        Ok(())
    }
}

fn target(record: &RecordRef) -> ExistingRecordTarget {
    match record {
        RecordRef::Entity(id) => ExistingRecordTarget::Entity(*id),
        RecordRef::Relation(id) => ExistingRecordTarget::Relation(*id),
    }
}

fn capture_records(
    request: &ExecutionRequest<'_, '_>,
    selected: &SelectedRelationalBranchRead<'_>,
) -> Result<ExecutionArray<RecordRef>, TransactionCommitError> {
    let mut count = 0usize;
    visit_records(request, selected, |_| {
        count = count.checked_add(1).ok_or_else(overflow)?;
        Ok(())
    })?;
    let mut records = ExecutionRequestArrayBuilder::allocate(*request, count).map_err(array)?;
    visit_records(request, selected, |record| {
        records.push(record).map_err(array)
    })?;
    records.seal().map_err(array)
}

fn visit_records(
    request: &ExecutionRequest<'_, '_>,
    selected: &SelectedRelationalBranchRead<'_>,
    mut visit: impl FnMut(RecordRef) -> Result<(), TransactionCommitError>,
) -> Result<(), TransactionCommitError> {
    let view = InvariantStateView::new(selected, selected.version_id());
    for partition in selected.selected().state().root().partition_ids() {
        // Native's occupied-slot iterators have fixed inline stacks and quote
        // zero heap bytes. Visit occupied slots, never the allocation frontier.
        for slot in view.try_entity_slots(partition, |work, _| charge(request, work))? {
            charge(request, 1)?;
            if let Some(metadata) = view.entity_metadata_for_slot(partition, slot) {
                visit(RecordRef::Entity(metadata.entity_id))?;
            }
        }
        for slot in view.try_relation_slots(partition, |work, _| charge(request, work))? {
            charge(request, 1)?;
            if let Some(metadata) = view.relation_metadata_for_slot(partition, slot) {
                visit(RecordRef::Relation(metadata.relation_id))?;
            }
        }
    }
    Ok(())
}

fn charge(request: &ExecutionRequest<'_, '_>, work: u64) -> Result<(), TransactionCommitError> {
    request
        .run_native_with_checkpoint(|context| {
            if context.has_accounting() {
                context.checkpoint(work)
            } else {
                context.poll()
            }
        })
        .map_err(|cause| TransactionCommitError::execution(cause.into()))?
        .0
        .map_err(|cause| {
            TransactionCommitError::execution(
                worth_execution::WorkCeilingDenial::Stopped(cause).into(),
            )
        })
}
#[cfg(test)]
#[path = "coverage_inline_type_budget.rs"]
pub(crate) mod inline_type_budget;
