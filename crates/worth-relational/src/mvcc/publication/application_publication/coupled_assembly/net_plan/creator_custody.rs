//! Full-generation allocation evidence fences destructive cross-writer suffixes.
use super::super::*;
use crate::{
    identity::data::EntityId,
    publication::data::{AuthoredRetirementRefusal as Cause, PublicationError},
    transactions::data::{EntityMutationIntent, MutationIntent, RecordRef, RelationMutationIntent},
};
use worth_execution::{ExecutionOrderedMap, ExecutionOrderedMapDenial};
type Writer = crate::publication::data::CoupledFreshWriter;
type Creator = (Writer, EntityId);
#[derive(Default)]
pub(in crate::mvcc::publication::application_publication::coupled_assembly) struct CreatorCustody {
    records: ExecutionOrderedMap<RecordRef, Creator>,
}
impl CreatorCustody {
    pub(in crate::mvcc::publication::application_publication::coupled_assembly) fn retain(
        &mut self,
        execution: &ExecutionRequest<'_, '_>,
        part: Writer,
        source: EntityId,
        outcome: &crate::authority::mutation::MutationApplyOutcome,
    ) -> Result<(), TransactionCommitError> {
        for id in outcome.created_entities.allocated_identities() {
            self.insert(execution, RecordRef::Entity(id), (part, source))?;
        }
        for (_, id) in outcome.created_relations.iter() {
            self.insert(execution, RecordRef::Relation(*id), (part, source))?;
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
                    crate::transactions::data::CommitConflict::new(
                        crate::transactions::data::ConflictClass::ExecutionAllocationDenied {
                            denial,
                        },
                    ),
                ),
                ExecutionOrderedMapDenial::CapacityOverflow => custody::execution_stop(
                    crate::execution::RelationalExecutionDenialCause::WorkCounterOverflow,
                ),
            })?;
        Ok(())
    }
    pub(in crate::mvcc::publication::application_publication::coupled_assembly) fn require(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        writer: Writer,
        source: EntityId,
        plan: &crate::transactions::data::MergedCommitPlan,
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
