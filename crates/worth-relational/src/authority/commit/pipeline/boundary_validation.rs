use super::draft_execution::PreparedCommitExecution;
use super::phase_backing::CommitPhaseBacking;

pub(super) struct BoundaryValidatedCommitExecution {
    prepared: PreparedCommitExecution,
    invariant_executions: Vec<crate::validation::engine::InvariantExecutionResult>,
}

impl BoundaryValidatedCommitExecution {
    pub(super) fn into_parts(
        self,
    ) -> (
        PreparedCommitExecution,
        Vec<crate::validation::engine::InvariantExecutionResult>,
    ) {
        (self.prepared, self.invariant_executions)
    }
}

pub(super) fn validate_commit_boundary(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    prepared: CommitPhaseBacking<PreparedCommitExecution>,
) -> Result<
    CommitPhaseBacking<BoundaryValidatedCommitExecution>,
    crate::transactions::data::outcomes::transaction_commit_error::TransactionCommitError,
> {
    let result = {
        let mut prepared = prepared.into_inner();
        super::consult_stage(execution, super::CommitStage::Boundary)?;
        let (admitted, ..) = prepared.boundary_parts();
        let invariant = admitted.take_prevalidated_invariants().ok_or_else(|| {
            crate::transactions::data::outcomes::transaction_commit_error::TransactionCommitError::conflict(
                crate::transactions::data::outcomes::commit_conflict::CommitConflict::new(
                    crate::transactions::data::outcomes::conflict_class::ConflictClass::StaleValidationBasis {
                        detail: "candidate is missing its proposed-state invariant evidence".into(),
                    },
                ),
            )
        })?;
        let (log, timing) = admitted.commit_phase_state();
        log.begin_phase(crate::transactions::data::commit_log::CommitPhase::InvariantPreCheck);
        log.record_invariant_outcomes(&invariant);
        log.complete_phase(crate::transactions::data::commit_log::CommitPhase::InvariantPreCheck);
        timing.invariant_pre_check_micros = 0;
        BoundaryValidatedCommitExecution {
            prepared,
            invariant_executions: vec![invariant],
        }
    };
    CommitPhaseBacking::new(execution, result)
}
