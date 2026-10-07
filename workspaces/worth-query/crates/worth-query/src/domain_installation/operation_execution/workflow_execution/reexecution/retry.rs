use crate::basis_lifecycle::BasisOperationLane;
use crate::domain_installation::WorthQueryConditionalProvenance;
use crate::identity::hash_parts;
use crate::runtime::WorthQueryWorkspace;

use super::workflow_progression_state::WorthQueryWorkflowAdvanceStep;
use super::{
    WorthQueryWorkflowAdvanceDenial, WorthQueryWorkflowAdvanceDenialKind,
    WorthQueryWorkflowIntentValue, WorthQueryWorkflowRun,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryStageAttemptPreparationDenial {
    StageRetryNotDeclaredIdempotent,
    ManagedArtifactRequiresWorkflowReexecution,
}

pub struct WorthQueryWorkflowStageAttempt<D, O, F, L: BasisOperationLane> {
    run: WorthQueryWorkflowRun<D, O, F, L>,
    stage_identity: String,
    input: WorthQueryWorkflowIntentValue,
    ordinal: u64,
    identity: String,
}

pub struct WorthQueryRetryableStageFailure<D, O, F, L: BasisOperationLane> {
    attempt: WorthQueryWorkflowStageAttempt<D, O, F, L>,
    denial: WorthQueryWorkflowAdvanceDenial,
}

pub struct WorthQueryDeferredStageAttempt<D, O, F, L: BasisOperationLane> {
    run: WorthQueryWorkflowRun<D, O, F, L>,
    conditional: Vec<WorthQueryConditionalProvenance>,
    attempt_identity: String,
}

#[derive(Debug)]
pub struct WorthQueryTerminalStageAttemptFailure {
    attempt_identity: String,
    denial: WorthQueryWorkflowAdvanceDenial,
}

pub enum WorthQueryWorkflowStageAttemptOutcome<D, O, F, L: BasisOperationLane> {
    Success(WorthQueryWorkflowRun<D, O, F, L>),
    Retryable(WorthQueryRetryableStageFailure<D, O, F, L>),
    Deferred(WorthQueryDeferredStageAttempt<D, O, F, L>),
    Failed(WorthQueryTerminalStageAttemptFailure),
}

impl<D: 'static, O: 'static, F: 'static, L: BasisOperationLane> WorthQueryWorkflowRun<D, O, F, L> {
    pub fn prepare_stage_attempt(
        self,
        stage_identity: impl Into<String>,
        input: WorthQueryWorkflowIntentValue,
    ) -> Result<WorthQueryWorkflowStageAttempt<D, O, F, L>, WorthQueryStageAttemptPreparationDenial>
    {
        if !self.executor.idempotent_stage_retry() {
            return Err(WorthQueryStageAttemptPreparationDenial::StageRetryNotDeclaredIdempotent);
        }
        if input.runtime_value().is_none() {
            return Err(
                WorthQueryStageAttemptPreparationDenial::ManagedArtifactRequiresWorkflowReexecution,
            );
        }
        Ok(WorthQueryWorkflowStageAttempt::new(
            self,
            stage_identity.into(),
            input,
            1,
        ))
    }
}

impl<D: 'static, O: 'static, F: 'static, L: BasisOperationLane>
    WorthQueryWorkflowStageAttempt<D, O, F, L>
{
    fn new(
        run: WorthQueryWorkflowRun<D, O, F, L>,
        stage_identity: String,
        input: WorthQueryWorkflowIntentValue,
        ordinal: u64,
    ) -> Self {
        let identity = hash_parts(&[
            "worth_query_workflow_stage_attempt_v1".into(),
            format!("run:{}", run.identity()),
            format!("stage:{stage_identity}"),
            format!("ordinal:{ordinal}"),
        ]);
        Self {
            run,
            stage_identity,
            input,
            ordinal,
            identity,
        }
    }

    pub fn identity(&self) -> &str {
        &self.identity
    }

    pub fn execute(
        mut self,
        workspace: &mut WorthQueryWorkspace,
        request: worth_execution::ExecutionRequest<'_, '_>,
    ) -> WorthQueryWorkflowStageAttemptOutcome<D, O, F, L> {
        match self.run.advance_once(
            &self.stage_identity,
            self.input
                .runtime_value()
                .expect("stage-attempt preparation rejected managed artifact intent"),
            workspace,
            request,
        ) {
            Ok(WorthQueryWorkflowAdvanceStep::Advanced) => {
                WorthQueryWorkflowStageAttemptOutcome::Success(self.run)
            }
            Ok(WorthQueryWorkflowAdvanceStep::Deferred(conditional)) => {
                WorthQueryWorkflowStageAttemptOutcome::Deferred(WorthQueryDeferredStageAttempt {
                    run: self.run,
                    conditional,
                    attempt_identity: self.identity,
                })
            }
            Err(denial)
                if denial.executed_effects().is_empty()
                    && matches!(
                        denial.kind(),
                        WorthQueryWorkflowAdvanceDenialKind::StageExecutor { .. }
                    ) =>
            {
                WorthQueryWorkflowStageAttemptOutcome::Retryable(WorthQueryRetryableStageFailure {
                    attempt: self,
                    denial,
                })
            }
            Err(denial) => WorthQueryWorkflowStageAttemptOutcome::Failed(
                WorthQueryTerminalStageAttemptFailure {
                    attempt_identity: self.identity,
                    denial,
                },
            ),
        }
    }
}

impl<D: 'static, O: 'static, F: 'static, L: BasisOperationLane>
    WorthQueryRetryableStageFailure<D, O, F, L>
{
    pub fn failed_attempt_identity(&self) -> &str {
        self.attempt.identity()
    }
    pub fn denial(&self) -> &WorthQueryWorkflowAdvanceDenial {
        &self.denial
    }
    pub fn retry(self) -> WorthQueryWorkflowStageAttempt<D, O, F, L> {
        let attempt = self.attempt;
        WorthQueryWorkflowStageAttempt::new(
            attempt.run,
            attempt.stage_identity,
            attempt.input,
            attempt.ordinal + 1,
        )
    }
}

impl<D, O, F, L: BasisOperationLane> WorthQueryDeferredStageAttempt<D, O, F, L> {
    pub fn attempt_identity(&self) -> &str {
        &self.attempt_identity
    }
    pub fn run(&self) -> &WorthQueryWorkflowRun<D, O, F, L> {
        &self.run
    }
    pub fn conditional_provenance(&self) -> &[WorthQueryConditionalProvenance] {
        &self.conditional
    }
}

impl WorthQueryTerminalStageAttemptFailure {
    pub fn attempt_identity(&self) -> &str {
        &self.attempt_identity
    }
    pub fn denial(&self) -> &WorthQueryWorkflowAdvanceDenial {
        &self.denial
    }
}
