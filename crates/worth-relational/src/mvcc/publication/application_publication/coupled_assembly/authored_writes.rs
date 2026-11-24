//! Native-applied authored writes obey one writer per record (D1612).
use super::assembly_failure::RelationalCoupledAssemblyFailure;
use crate::mvcc::publication::application_publication::custody;
use crate::transactions::data::intents::mutation_intent::EntityMutationIntent;
use crate::transactions::data::intents::mutation_intent::MutationIntent;
use crate::transactions::data::intents::mutation_intent::RelationMutationIntent;
use crate::transactions::data::outcomes::commit_conflict::CommitConflict;
use crate::transactions::data::outcomes::conflict_class::ConflictClass;
use crate::transactions::data::outcomes::plan_artifacts::MergedCommitPlan;
use crate::transactions::data::outcomes::transaction_commit_error::TransactionCommitError;
use crate::transactions::data::primitives::ExistingRecordTarget;
use crate::transactions::data::primitives::RecordRef;
use worth_execution::ExecutionRequest;
use worth_execution::{ExecutionOrderedMap, ExecutionOrderedMapDenial};

#[derive(Default)]
pub(super) struct AuthoredWrites {
    writers: ExecutionOrderedMap<RecordRef, usize>,
}
impl AuthoredWrites {
    /// O((intents + changed records) log(total authored records)). The command
    /// targets retain even no-op writes; applied records also include births.
    pub(super) fn retain(
        &mut self,
        execution: &ExecutionRequest<'_, '_>,
        stage: usize,
        plan: &MergedCommitPlan,
        records: &[RecordRef],
    ) -> Result<(), RelationalCoupledAssemblyFailure> {
        for record in plan
            .merged_intents
            .iter()
            .filter_map(write_target)
            .chain(records.iter().cloned())
        {
            if let Some(earlier) = self.writers.get(&record) {
                if *earlier != stage {
                    let target = match record {
                        RecordRef::Entity(id) => ExistingRecordTarget::Entity(id),
                        RecordRef::Relation(id) => ExistingRecordTarget::Relation(id),
                    };
                    let mut failure =
                        RelationalCoupledAssemblyFailure::from(TransactionCommitError::conflict(
                            CommitConflict::new(ConflictClass::ConflictingIntent { target }),
                        ));
                    failure.stage_conflict = Some((record, *earlier, stage));
                    return Err(failure);
                }
                continue;
            }
            self.writers
                .insert_cloned_native_for_request(&record, &stage, *execution)
                .map_err(|cause| match cause {
                    ExecutionOrderedMapDenial::Request(cause) => custody::execution_stop(cause),
                    ExecutionOrderedMapDenial::Allocation(denial) => {
                        TransactionCommitError::conflict(CommitConflict::new(
                            ConflictClass::ExecutionAllocationDenied { denial },
                        ))
                    }
                    ExecutionOrderedMapDenial::CapacityOverflow => custody::execution_stop(
                        crate::execution::RelationalExecutionDenialCause::WorkCounterOverflow,
                    ),
                })?;
        }
        Ok(())
    }
}
fn write_target(intent: &MutationIntent) -> Option<RecordRef> {
    // Revalidation judges a record but authors no write to it.
    if matches!(
        intent,
        MutationIntent::Entity(EntityMutationIntent::Revalidate(_))
            | MutationIntent::Relation(RelationMutationIntent::Revalidate(_))
    ) {
        return None;
    }
    intent.existing_record_target().map(|target| match target {
        ExistingRecordTarget::Entity(id) => RecordRef::Entity(id),
        ExistingRecordTarget::Relation(id) => RecordRef::Relation(id),
    })
}
