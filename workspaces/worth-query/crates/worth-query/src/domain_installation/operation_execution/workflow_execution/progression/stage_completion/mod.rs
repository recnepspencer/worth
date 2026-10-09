use crate::basis_lifecycle::BasisOperationLane;
use crate::runtime::WorthQueryWorkspace;

mod entry;
pub(in crate::domain_installation::operation_execution) mod evidence_validation;
mod execution_context;
mod outcome_routing;
use super::workflow_parallel_progression::frontier::CanonicalWorkflowStageResult;

use super::workflow_graph_execution::invoke_stage_graphs;
use super::workflow_progression_state::{
    WorthQueryExecutedWorkflowStage, WorthQueryStageConditionAdmission,
    WorthQueryWorkflowAdvanceStep,
};
use super::{
    WorthQueryAdmittedWorkflowStage, WorthQueryBoundGraphExecutionReceipt,
    WorthQueryWorkflowAdvanceDenial, WorthQueryWorkflowAdvanceDenialKind,
    WorthQueryWorkflowInvariantOutcome, WorthQueryWorkflowRun, WorthQueryWorkflowRunCounters,
    WorthQueryWorkflowSemanticValue, WorthQueryWorkflowStageExecutionAuthority,
    WorthQueryWorkflowStageExecutionContext, WorthQueryWorkflowStageExecutionScope,
    WorthQueryWorkflowStageMaterialParts, WorthQueryWorkflowStageReceipt,
    WorthQueryWorkflowStageRuntimeAdmission, WorthQueryWorkflowValue,
};
use worth_proof::TransitionOutcome;

pub type WorthQueryWorkflowAdvanceOutcome<D, O, F, L> = TransitionOutcome<
    WorthQueryWorkflowRun<D, O, F, L>,
    WorthQueryWorkflowAdvanceDenial,
    crate::domain_installation::WorthQueryDeferredWorkflowStage<D, O, F, L>,
    WorthQueryWorkflowAdvanceDenial,
    WorthQueryWorkflowAdvanceDenial,
    WorthQueryWorkflowAdvanceDenial,
>;

struct WorthQueryWorkflowStageCompletion {
    semantic_input: super::WorthQueryWorkflowSemanticValue,
    material: WorthQueryWorkflowStageMaterialParts,
    graph_receipts: Vec<WorthQueryBoundGraphExecutionReceipt>,
    conditional: Vec<crate::domain_installation::WorthQueryConditionalProvenance>,
    execution_snapshot: crate::memory_workspace::WorthQuerySnapshotIdentity,
    effect_workflow_binding: crate::workflow::WorkflowContextBinding,
    counters_before: super::WorthQueryWorkflowRunCounters,
    resource_evidence: super::WorthQueryExecutionResourceAttemptEvidence,
}

impl WorthQueryWorkflowStageCompletion {
    fn denial(
        &self,
        counters: super::WorthQueryWorkflowRunCounters,
        kind: WorthQueryWorkflowAdvanceDenialKind,
    ) -> WorthQueryWorkflowAdvanceDenial {
        WorthQueryWorkflowAdvanceDenial::with_executed_effects(
            kind,
            counters,
            self.material.executed_effects.clone(),
        )
        .with_graph_receipts(self.graph_receipts.clone())
    }
}

impl<D: 'static, O: 'static, F: 'static, L: BasisOperationLane> WorthQueryWorkflowRun<D, O, F, L> {
    pub(super) fn advance_once(
        &mut self,
        execution: &worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase<'_>,
        stage_identity: &str,
        input: WorthQueryWorkflowValue,
        workspace: &mut WorthQueryWorkspace,
    ) -> Result<WorthQueryWorkflowAdvanceStep, WorthQueryWorkflowAdvanceDenial> {
        let admission = self.admit_stage_runtime_authority(workspace)?;
        self.advance_once_with_runtime_admission(
            execution,
            stage_identity,
            input,
            workspace,
            admission,
        )
    }
    pub(super) fn advance_once_with_computation(
        &mut self,
        execution: &worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase<'_>,
        slot: CanonicalWorkflowStageResult,
        workspace: &mut WorthQueryWorkspace,
    ) -> Result<WorthQueryWorkflowAdvanceStep, WorthQueryWorkflowAdvanceDenial> {
        let runtime_admission = self.admit_stage_runtime_authority(workspace)?;
        let admitted = self.admit_stage(slot.stage_identity(), slot.input(), runtime_admission)?;
        self.advance_once_with_admitted_computation(execution, admitted, slot, workspace)
    }
    fn advance_once_with_runtime_admission(
        &mut self,
        execution: &worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase<'_>,
        stage_identity: &str,
        input: WorthQueryWorkflowValue,
        workspace: &mut WorthQueryWorkspace,
        runtime_admission: WorthQueryWorkflowStageRuntimeAdmission,
    ) -> Result<WorthQueryWorkflowAdvanceStep, WorthQueryWorkflowAdvanceDenial> {
        let admitted = self.admit_stage(stage_identity, &input, runtime_admission)?;
        self.advance_once_with_admitted_stage(execution, admitted, input, workspace)
    }

    pub(super) fn advance_with_admitted_stage(
        mut self,
        execution: &worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase<'_>,
        admitted: WorthQueryAdmittedWorkflowStage,
        input: WorthQueryWorkflowValue,
        workspace: &mut WorthQueryWorkspace,
    ) -> WorthQueryWorkflowAdvanceOutcome<D, O, F, L> {
        match self.advance_once_with_admitted_stage(execution, admitted, input, workspace) {
            Ok(WorthQueryWorkflowAdvanceStep::Advanced) => TransitionOutcome::Success(self),
            Ok(WorthQueryWorkflowAdvanceStep::Deferred(conditional)) => {
                TransitionOutcome::Deferred(
                    crate::domain_installation::WorthQueryDeferredWorkflowStage {
                        run: self,
                        conditional,
                    },
                )
            }
            Err(denial) => self.outcome_from_denial(denial),
        }
    }

    fn advance_once_with_admitted_stage(
        &mut self,
        execution: &worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase<'_>,
        admitted: WorthQueryAdmittedWorkflowStage,
        input: WorthQueryWorkflowValue,
        workspace: &mut WorthQueryWorkspace,
    ) -> Result<WorthQueryWorkflowAdvanceStep, WorthQueryWorkflowAdvanceDenial> {
        let prepared =
            self.prepare_frontier_computation(vec![(admitted.stage.identity().into(), input)]);
        prepared
            .compute(
                execution
                    .execution_request_for(&workspace.advancement_owner())
                    .map_err(|cause| {
                        self.denial(WorthQueryWorkflowAdvanceDenialKind::ExecutionRequest(
                            cause.into(),
                        ))
                    })?,
            )
            .apply(execution, self, workspace, Some(admitted))
    }
    pub(super) fn advance_once_with_admitted_computation(
        &mut self,
        execution: &worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase<'_>,
        admitted: WorthQueryAdmittedWorkflowStage,
        slot: CanonicalWorkflowStageResult,
        workspace: &mut WorthQueryWorkspace,
    ) -> Result<WorthQueryWorkflowAdvanceStep, WorthQueryWorkflowAdvanceDenial> {
        let WorthQueryAdmittedWorkflowStage {
            stage,
            counters_before,
        } = admitted;
        let (resources, resource_evidence) = self
            .managed_run()
            .stage_resources_and_evidence(stage.identity())
            .ok_or_else(|| {
                WorthQueryWorkflowAdvanceDenial::new(
                    WorthQueryWorkflowAdvanceDenialKind::ResourceAdmissionMissing,
                    self.counters,
                )
            })?;
        let semantic_input = slot.input().semantic_value();
        let graph_snapshot = workspace.snapshot_identity();
        let conditional = match self.admit_stage_condition(
            execution,
            stage.identity(),
            &resources,
            &resource_evidence,
            &graph_snapshot,
            workspace,
        )? {
            WorthQueryStageConditionAdmission::Admitted(conditional) => conditional,
            WorthQueryStageConditionAdmission::Deferred(conditional) => {
                return Ok(WorthQueryWorkflowAdvanceStep::Deferred(conditional));
            }
        };
        let managed = self
            .managed
            .take()
            .expect("live workflow owns its managed run");
        let (managed, graph_receipts) = invoke_stage_graphs(
            execution,
            &self.bound,
            managed,
            &self.identity,
            &stage,
            &mut self.counters,
        )?;
        self.managed = Some(managed);
        let executed = self.execute_admitted_stage(
            execution,
            &stage,
            &resources,
            &resource_evidence,
            slot,
            &graph_receipts,
            workspace,
        )?;
        let evidence = self.validate_stage_evidence(
            &stage,
            WorthQueryWorkflowStageCompletion {
                semantic_input,
                material: executed.material,
                graph_receipts,
                conditional,
                execution_snapshot: workspace.snapshot_identity(),
                effect_workflow_binding: executed.effect_workflow_binding,
                counters_before,
                resource_evidence,
            },
        )?;
        self.retain_admitted_stage(&stage, executed.predecessor_receipt_identities, evidence);
        Ok(WorthQueryWorkflowAdvanceStep::Advanced)
    }

    fn admit_stage_condition(
        &mut self,
        execution: &worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase<'_>,
        stage_identity: &str,
        resources: &super::WorthQueryAdmittedExecutionResourcePlan,
        resource_evidence: &super::WorthQueryExecutionResourceAttemptEvidence,
        graph_snapshot: &crate::memory_workspace::WorthQuerySnapshotIdentity,
        workspace: &mut WorthQueryWorkspace,
    ) -> Result<WorthQueryStageConditionAdmission, WorthQueryWorkflowAdvanceDenial> {
        match super::workflow_conditional_stage_evaluation::evaluate(
            execution
                .execution_request_for(&workspace.advancement_owner())
                .map_err(|cause| {
                    self.denial(WorthQueryWorkflowAdvanceDenialKind::ExecutionRequest(
                        cause.into(),
                    ))
                })?,
            &self.bound,
            workspace,
            graph_snapshot,
            stage_identity,
            &self.identity,
            self.receipts.len() as u64 + 1,
            resources,
            resource_evidence,
            &mut self.counters,
        ) {
            Ok(conditional) => Ok(WorthQueryStageConditionAdmission::Admitted(conditional)),
            Err(super::workflow_conditional_stage_evaluation::ConditionalStageStop::Deferred(
                conditional,
            )) => Ok(WorthQueryStageConditionAdmission::Deferred(conditional)),
            Err(super::workflow_conditional_stage_evaluation::ConditionalStageStop::Denied(
                kind,
            )) => Err(WorthQueryWorkflowAdvanceDenial::new(kind, self.counters)),
        }
    }

    fn execute_admitted_stage(
        &mut self,
        execution: &worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase<'_>,
        stage: &worth_query_installation::facade::WorthQueryPortableWorkflowStage,
        resources: &super::WorthQueryAdmittedExecutionResourcePlan,
        resource_evidence: &super::WorthQueryExecutionResourceAttemptEvidence,
        slot: CanonicalWorkflowStageResult,
        graph_receipts: &[WorthQueryBoundGraphExecutionReceipt],
        workspace: &mut WorthQueryWorkspace,
    ) -> Result<WorthQueryExecutedWorkflowStage, WorthQueryWorkflowAdvanceDenial> {
        let predecessor_indices = self.predecessor_receipt_indices(stage)?;
        let predecessor_receipts = predecessor_indices
            .iter()
            .map(|index| &self.receipts[*index])
            .collect::<Vec<_>>();
        self.assert_predecessor_authority(&predecessor_receipts);
        self.counters.stage_executor_contacts += 1;
        let effect_workflow_binding =
            self.stage_effect_workflow_binding(stage, workspace.snapshot_identity());
        let context = self.stage_execution_context(
            execution
                .execution_request_for(&workspace.advancement_owner())
                .map_err(|cause| {
                    self.denial(WorthQueryWorkflowAdvanceDenialKind::ExecutionRequest(
                        cause.into(),
                    ))
                })?,
            stage,
            &predecessor_receipts,
            graph_receipts,
            resources,
            resource_evidence,
            effect_workflow_binding.clone(),
        )?;
        let material = self
            .executor
            .apply(slot, &context, workspace)
            .map_err(|failure| {
                let class = failure.class().clone();
                let kind = if stage.semantics().failure_classes.contains(&class) {
                    WorthQueryWorkflowAdvanceDenialKind::StageExecutor {
                        class,
                        detail: failure.detail().into(),
                    }
                } else {
                    WorthQueryWorkflowAdvanceDenialKind::UndeclaredFailureClass(class)
                };
                WorthQueryWorkflowAdvanceDenial::with_executed_effects(
                    kind,
                    self.counters,
                    failure.executed_effects().to_vec(),
                )
                .with_graph_receipts(graph_receipts.to_vec())
            })?;
        Ok(WorthQueryExecutedWorkflowStage {
            predecessor_receipt_identities: predecessor_receipts
                .iter()
                .map(|receipt| receipt.identity().to_string())
                .collect(),
            material: material.into_parts(),
            effect_workflow_binding,
        })
    }
}
