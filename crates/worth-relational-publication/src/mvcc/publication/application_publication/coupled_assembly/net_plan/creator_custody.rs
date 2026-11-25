//! Full-generation allocation evidence fences destructive cross-writer suffixes.
use crate::mvcc::publication::application_publication::custody;
use contracts::execution::request_stage::checkpoint;
use contracts::identity::data::record_ids::EntityId;
use contracts::publication::data::authored_retirement_refusal::AuthoredRetirementRefusal as Cause;
use contracts::publication::data::publication_error::PublicationError;
use contracts::publication::data::publication_error::PublicationErrorOwnerPort as _;
use contracts::transactions::data::intents::mutation_intent::EntityMutationIntent;
use contracts::transactions::data::intents::mutation_intent::MutationIntent;
use contracts::transactions::data::intents::mutation_intent::RelationMutationIntent;
use contracts::transactions::data::outcomes::commit_conflict::CommitConflictOwnerPort as _;
use contracts::transactions::data::outcomes::transaction_commit_error::TransactionCommitError;
use contracts::transactions::data::primitives::RecordRef;
use worth_execution::ExecutionRequest;
use worth_execution::{ExecutionOrderedMap, ExecutionOrderedMapDenial};
use worth_relational_contracts::internal_port as contracts;
type Writer = contracts::publication::data::authored_retirement_refusal::CoupledFreshWriter;
type Creator = (Writer, EntityId);
#[derive(Default)]
pub struct CreatorCustody {
    records: ExecutionOrderedMap<RecordRef, Creator>,
}
impl CreatorCustody {
    pub fn retain(
        &mut self,
        execution: &ExecutionRequest<'_, '_>,
        part: Writer,
        source: EntityId,
        outcome: &worth_relational_mutation::internal_port::authority::mutation::execution::MutationApplyOutcome,
    ) -> Result<(), TransactionCommitError> {
        for id in outcome.created_entities.allocated_identities() {
            self.insert(execution, RecordRef::Entity(id), (part, source))?;
        }
        for (_, id) in outcome.created_relations.iter() {
            self.insert(execution, RecordRef::Relation(*id), (part, source))?;
        }
        Ok(())
    }
    /// Forget a retracted round's fresh records: no live writer created them.
    /// O(|records| · log n).
    pub fn forget<'r>(
        &mut self,
        execution: &ExecutionRequest<'_, '_>,
        records: impl IntoIterator<Item = &'r RecordRef>,
    ) -> Result<(), TransactionCommitError> {
        for record in records {
            self.records
                .remove_for_request(record, *execution)
                .map_err(|cause| match cause {
                    ExecutionOrderedMapDenial::Request(cause) => custody::execution_stop(cause),
                    _ => custody::execution_stop(
                        contracts::execution::denial_cause::RelationalExecutionDenialCause::AllocationFailed,
                    ),
                })?;
        }
        Ok(())
    }
    fn insert(
        &mut self,
        execution: &ExecutionRequest<'_, '_>,
        record: RecordRef,
        creator: Creator,
    ) -> Result<(), TransactionCommitError> {
        self.records
            .insert_cloned_native_for_request(&record, &creator, *execution)
            .map_err(|cause| match cause {
                ExecutionOrderedMapDenial::Request(cause) => custody::execution_stop(cause),
                ExecutionOrderedMapDenial::Allocation(denial) => TransactionCommitError::conflict(
                    contracts::transactions::data::outcomes::commit_conflict::CommitConflict::new(
                        contracts::transactions::data::outcomes::conflict_class::ConflictClass::ExecutionAllocationDenied {
                            denial,
                        },
                    ),
                ),
                ExecutionOrderedMapDenial::CapacityOverflow => custody::execution_stop(
                    contracts::execution::denial_cause::RelationalExecutionDenialCause::WorkCounterOverflow,
                ),
            })?;
        Ok(())
    }
    pub fn require(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        writer: Writer,
        source: EntityId,
        plan: &contracts::transactions::data::outcomes::plan_artifacts::MergedCommitPlan,
    ) -> Result<(), TransactionCommitError> {
        execution
            .run_native_with_checkpoint(|context| {
                for intent in &plan.merged_intents {
                    checkpoint(
                        context,
                        u64::from(usize::BITS - self.records.len().max(1).leading_zeros()) * 11 * 4
                            + 1,
                    )
                    .map_err(custody::execution_stop)?;
                    let target = match intent {
                        MutationIntent::Entity(EntityMutationIntent::Delete(spec)) => {
                            RecordRef::Entity(spec.entity_id)
                        }
                        MutationIntent::Entity(EntityMutationIntent::Replace(spec)) => {
                            RecordRef::Entity(spec.entity_id)
                        }
                        MutationIntent::Relation(RelationMutationIntent::Delete(spec)) => {
                            RecordRef::Relation(spec.relation_id)
                        }
                        MutationIntent::Relation(RelationMutationIntent::UpdateEndpoints(spec)) => {
                            RecordRef::Relation(spec.relation_id)
                        }
                        _ => continue,
                    };
                    if let Some((creator, owner)) = self.records.get(&target)
                        && *creator != writer
                    {
                        let mut error = PublicationError::authored_retirement(
                            Cause::ForeignFreshCoupledRecord {
                                creator: *creator,
                                writer,
                                owner: *owner,
                                source,
                            },
                            target.clone(),
                        );
                        error.context = error.context.with_records(vec![
                            target,
                            RecordRef::Entity(*owner),
                            RecordRef::Entity(source),
                        ]);
                        return Err(TransactionCommitError::publication(error));
                    }
                }
                Ok(())
            })
            .map_err(custody::execution_stop)?
            .0
    }
}
