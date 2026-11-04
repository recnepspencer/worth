//! Every Native participant must retain its actual Query settlement session.
use super::*;
use crate::domain_computation::WorthQueryInvariantExecutionFailure;
use worth_relational::facade::{identity::EntityId, mvcc::ValidatedRelationalProposal};
pub(super) fn require_complete(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    candidate: &ValidatedRelationalProposal,
    members: &[CoupledApplicationMember],
) -> Result<(), WorthQueryInvariantExecutionFailure> {
    execution
        .run_native_with_checkpoint(|context| {
            for (producer, meaning) in candidate.coupled_participants() {
                for member in members {
                    context
                        .checkpoint(
                            (producer.len()
                                + member.producer.len()
                                + meaning.family.len()
                                + member.meaning.family.len()
                                + 97) as u64,
                        )
                        .map_err(|cause| {
                            super::invariant_execution_failure::map_validation_failure(native_stop(
                                cause.into(),
                            ))
                        })?;
                }
                if !members
                    .iter()
                    .any(|member| member.producer == producer && member.meaning == *meaning)
                {
                    return Err(missing(meaning.source_scope));
                }
            }
            Ok(())
        })
        .map_err(|cause| {
            super::invariant_execution_failure::map_validation_failure(native_stop(cause.into()))
        })?
        .0
}
fn missing(source: EntityId) -> WorthQueryInvariantExecutionFailure {
    WorthQueryInvariantExecutionFailure::missing_coupled_settlement(source)
}

fn native_stop(
    cause: worth_relational::facade::transactions::RelationalExecutionDenialCause,
) -> worth_relational::facade::mvcc::TransactionCommitError {
    worth_relational::facade::mvcc::TransactionCommitError::Execution {
        denial: worth_relational::facade::transactions::CommitExecutionDenial {
            kind: worth_relational::facade::transactions::CommitExecutionDenialKind::Cause(cause),
            partition_identity: None,
        },
        context: worth_relational::facade::errors::ErrorContext::new(
            worth_relational::facade::errors::RelationalSubsystem::Transaction,
            worth_relational::facade::errors::ErrorOperation::Validate,
        ),
        commit_log: worth_relational::facade::transactions::CommitLog::new(),
    }
}
