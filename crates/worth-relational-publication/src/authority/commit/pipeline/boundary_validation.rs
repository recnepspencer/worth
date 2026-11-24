use super::draft_execution::PreparedCommitExecution;
use super::phase_backing::CommitPhaseBacking;
use contracts::transactions::data::outcomes::commit_conflict::CommitConflictOwnerPort as _;
use worth_relational_contracts::internal_port as contracts;

pub(super) struct BoundaryValidatedCommitExecution {
    prepared: PreparedCommitExecution,
    invariant_executions: Vec<worth_relational_mutation::internal_port::validation::engine::result::InvariantExecutionResult>,
}

impl BoundaryValidatedCommitExecution {
    pub(super) fn into_parts(
        self,
    ) -> (
        PreparedCommitExecution,
        Vec<worth_relational_mutation::internal_port::validation::engine::result::InvariantExecutionResult>,
    ){
        (self.prepared, self.invariant_executions)
    }
}

pub(super) fn validate_commit_boundary(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    prepared: CommitPhaseBacking<PreparedCommitExecution>,
) -> Result<
    CommitPhaseBacking<BoundaryValidatedCommitExecution>,
    contracts::transactions::data::outcomes::transaction_commit_error::TransactionCommitError,
> {
    let result = {
        let mut prepared = prepared.into_inner();
        super::consult_stage(execution, super::CommitStage::Boundary)?;
        let (admitted, ..) = prepared.boundary_parts();
        let invariant = admitted.take_prevalidated_invariants().ok_or_else(|| {
            contracts::transactions::data::outcomes::transaction_commit_error::TransactionCommitError::conflict(
                contracts::transactions::data::outcomes::commit_conflict::CommitConflict::new(
                    contracts::transactions::data::outcomes::conflict_class::ConflictClass::StaleValidationBasis {
                        detail: "candidate is missing its proposed-state invariant evidence".into(),
                    },
                ),
            )
        })?;
        let (log, timing) = admitted.commit_phase_state();
        log.begin_phase(contracts::transactions::data::commit_log::CommitPhase::InvariantPreCheck);
        log.record_invariant_outcomes(&invariant);
        log.complete_phase(
            contracts::transactions::data::commit_log::CommitPhase::InvariantPreCheck,
        );
        timing.invariant_pre_check_micros = 0;
        BoundaryValidatedCommitExecution {
            prepared,
            invariant_executions: vec![invariant],
        }
    };
    CommitPhaseBacking::new(execution, result)
}
