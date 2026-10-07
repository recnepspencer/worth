use super::super::workflow_parallel_progression::frontier::CanonicalWorkflowStageResult;
use super::super::{
    WorthQueryDomainWorkflowStageExecutor, WorthQueryExecutableDomainOperation,
    WorthQueryWorkflowOperation, WorthQueryWorkflowStageApplication,
    WorthQueryWorkflowStageExecutionContext, WorthQueryWorkflowStageExecutorFailure,
    WorthQueryWorkflowStageMaterial, WorthQueryWorkflowStageWorkspace,
};
use super::{ErasedReplaySemanticComparator, ErasedWorkflowStageExecutor};
use crate::runtime::WorthQueryWorkspace;
use std::marker::PhantomData;
use std::sync::Arc;

type WorkflowExecutorMarker<D, O, F> = fn() -> (D, O, F);
pub(super) struct TypedWorkflowStageExecutor<D, O, F, E> {
    pub(super) executor: Arc<E>,
    pub(super) marker: PhantomData<WorkflowExecutorMarker<D, O, F>>,
}

impl<D, O, F, E: WorthQueryDomainWorkflowStageExecutor<D, O, F>> ErasedWorkflowStageExecutor
    for TypedWorkflowStageExecutor<D, O, F, E>
where
    O: WorthQueryExecutableDomainOperation<D, F, Execution = WorthQueryWorkflowOperation>,
{
    fn idempotent_stage_retry(&self) -> bool {
        E::IDEMPOTENT_STAGE_RETRY
    }

    fn apply(
        &self,
        slot: CanonicalWorkflowStageResult,
        context: &WorthQueryWorkflowStageExecutionContext<'_>,
        workspace: &mut WorthQueryWorkspace,
    ) -> Result<WorthQueryWorkflowStageMaterial, WorthQueryWorkflowStageExecutorFailure> {
        let mut workspace = WorthQueryWorkflowStageWorkspace::new(
            workspace,
            context.artifact_production_authority(),
            context.artifact_access_authority(),
        );
        let (input, computed) = slot.into_parts();
        let mut material = match computed.and_then(|computed| {
            self.executor.apply(WorthQueryWorkflowStageApplication::new(
                input,
                computed,
                context,
                &mut workspace,
            ))
        }) {
            Ok(material) => material,
            Err(failure) => {
                return Err(failure.with_executed_effects(workspace.into_executed_effects()));
            }
        };
        if workspace.installed_read_executions() != usize::from(context.requires_primary_read()) {
            return Err(WorthQueryWorkflowStageExecutorFailure::new(
                crate::domain_installation::WorthQueryOperationFailureClass::Indeterminate,
                "workflow stage did not use its installed primary read exactly once",
            )
            .with_executed_effects(workspace.into_executed_effects()));
        }
        material.retain_query_executed_effects(workspace.into_executed_effects());
        Ok(material)
    }
}

impl<D, O, F, E> ErasedReplaySemanticComparator for TypedWorkflowStageExecutor<D, O, F, E>
where
    E: WorthQueryDomainWorkflowStageExecutor<D, O, F>
        + super::super::WorthQueryDomainReplaySemanticComparator<D, O, F>,
    O: WorthQueryExecutableDomainOperation<D, F, Execution = WorthQueryWorkflowOperation>,
{
    fn compare(
        &self,
        original: &super::super::WorthQueryWorkflowTraceSemantics,
        replay: &super::super::WorthQueryWorkflowTraceSemantics,
        noise: super::super::WorthQueryReplayNoiseContract,
    ) -> super::super::WorthQueryReplayComparison {
        self.executor
            .compare_replay_semantics(original, replay, noise)
    }
}
