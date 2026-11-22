//! Every Native participant must retain its actual Query settlement session.
use super::*;
use crate::domain_computation::primary_graph::admitted_capture::checkpoint;
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
                    checkpoint(
                        context,
                        (producer.len()
                            + member.producer_route().map_or(0, |(name, _)| name.len())
                            + meaning.family.len()
                            + member
                                .producer_route()
                                .map_or(0, |(_, meaning)| meaning.family.len())
                            + 97) as u64,
                    )
                    .map_err(|cause| {
                        super::invariant_execution_failure::map_validation_failure(native_stop(
                            cause.into(),
                        ))
                    })?;
                }
                if !members.iter().any(|member| {
                    member
                        .producer_route()
                        .is_some_and(|(name, actual)| name == producer && actual == meaning)
                }) {
                    return Err(missing(meaning.source_scope));
                }
            }
            for member in members {
                if let CoupledApplicationMemberKind::AuthoredStage { stage, source, .. } =
                    &member.kind
                {
                    checkpoint(context, 1).map_err(|cause| {
                        super::invariant_execution_failure::map_validation_failure(native_stop(
                            cause.into(),
                        ))
                    })?;
                    if candidate
                        .authored_stage_contribution_records(*stage, *source)
                        .is_none()
                    {
                        return Err(missing(*source));
                    }
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
