//! Consuming direct-execution phases before exact evidence completion.

use crate::basis_lifecycle::BasisOperationLane;
use worth_proof::TransitionOutcome;

use super::super::super::{
    WorthQueryAdmittedDirectOperation, WorthQueryBoundExecutionDenial,
    WorthQueryBoundExecutionDenialKind, WorthQueryBoundGraphExecutionReceipt,
    WorthQueryExecutableDomainOperation, WorthQueryOperationExecutionContext,
    WorthQueryOperationExecutionCounters,
};
use super::WorthQueryValidatedDirectEvidenceCompletion;

struct WorthQueryPreparedDirectExecution<D, O, F, L>
where
    L: BasisOperationLane,
    O: WorthQueryExecutableDomainOperation<D, F>,
{
    bound: crate::domain_installation::WorthQueryBoundDomainOperation<D, O, F, L>,
    input: O::Input,
    executor: std::sync::Arc<crate::domain_installation::WorthQueryInstalledDomainOperationExecutor>,
    phase_proof: crate::domain_installation::operation_authority_chain::WorthQueryOperationPhaseProof<
        crate::domain_installation::operation_authority_chain::WorthQueryResourceAdmittedOperationPhase,
    >,
    running: Option<worth_query_execution::facade::runtime::WorthQueryRunningDirectRun>,
    resources: super::super::super::WorthQueryAdmittedExecutionResourcePlan,
    execution_snapshot: crate::memory_workspace::WorthQuerySnapshotIdentity,
    conditional: Vec<crate::domain_installation::WorthQueryConditionalProvenance>,
    resource_evidence: super::super::super::WorthQueryExecutionResourceAttemptEvidence,
    counters: WorthQueryOperationExecutionCounters,
}

struct WorthQueryGraphCompletedDirectExecution<D, O, F, L>
where
    L: BasisOperationLane,
    O: WorthQueryExecutableDomainOperation<D, F>,
{
    prepared: WorthQueryPreparedDirectExecution<D, O, F, L>,
    graph_receipts: Vec<WorthQueryBoundGraphExecutionReceipt>,
}

struct WorthQueryExecutorCompletedDirectExecution<D, O, F, L, Output>
where
    L: BasisOperationLane,
    O: WorthQueryExecutableDomainOperation<D, F>,
{
    bound: crate::domain_installation::WorthQueryBoundDomainOperation<D, O, F, L>,
    phase_proof: crate::domain_installation::operation_authority_chain::WorthQueryOperationPhaseProof<
        crate::domain_installation::operation_authority_chain::WorthQueryResourceAdmittedOperationPhase,
    >,
    running: worth_query_execution::facade::runtime::WorthQueryRunningDirectRun,
    resources: super::super::super::WorthQueryAdmittedExecutionResourcePlan,
    output: Output,
    result_state: crate::domain_installation::WorthQueryOperationResultState,
    warnings: Vec<super::super::super::WorthQueryOperationExecutionWarning>,
    material: Option<super::super::super::WorthQueryDomainEvidenceMaterial>,
    graph_receipts: Vec<WorthQueryBoundGraphExecutionReceipt>,
    snapshot: crate::memory_workspace::WorthQuerySnapshotIdentity,
    conditional: Vec<crate::domain_installation::WorthQueryConditionalProvenance>,
    resource_evidence: super::super::super::WorthQueryExecutionResourceAttemptEvidence,
    counters: WorthQueryOperationExecutionCounters,
}

impl<D: 'static, O, F: 'static, L: BasisOperationLane> WorthQueryAdmittedDirectOperation<D, O, F, L>
where
    O: WorthQueryExecutableDomainOperation<
        D,
        F,
        Execution = super::super::super::WorthQueryDirectOperation,
    >,
{
    pub fn execute(
        self,
        workspace: &mut crate::runtime::WorthQueryWorkspace,
    ) -> super::super::WorthQueryBoundExecutionOutcome<D, O, F, L, O::Output> {
        let prepared = match WorthQueryPreparedDirectExecution::prepare(self, workspace) {
            Ok(prepared) => prepared,
            Err(outcome) => return outcome,
        };
        match prepared.invoke_graphs() {
            Ok(completed) => completed.invoke_executor(workspace),
            Err(outcome) => outcome,
        }
    }
}

impl<D: 'static, O, F: 'static, L: BasisOperationLane> WorthQueryPreparedDirectExecution<D, O, F, L>
where
    O: WorthQueryExecutableDomainOperation<
        D,
        F,
        Execution = super::super::super::WorthQueryDirectOperation,
    >,
{
    fn prepare(
        admitted: WorthQueryAdmittedDirectOperation<D, O, F, L>,
        workspace: &mut crate::runtime::WorthQueryWorkspace,
    ) -> Result<Self, super::super::WorthQueryBoundExecutionOutcome<D, O, F, L, O::Output>> {
        let mut counters = WorthQueryOperationExecutionCounters {
            runtime_authority_checks: 1,
            ..Default::default()
        };
        let witness =
            crate::domain_installation::WorthQueryInstalledDomainAuthorityWitness::from_authority(
                std::sync::Arc::clone(admitted.bound.operation().domain_authority()),
            );
        if let Err(denial) = workspace.validate_installed_domain_witness::<D>(&witness) {
            return Err(TransitionOutcome::Stale(
                WorthQueryBoundExecutionDenial::new(
                    WorthQueryBoundExecutionDenialKind::RuntimeAuthority(denial.kind()),
                    format!("{denial:?}"),
                    counters,
                ),
            ));
        }
        let resource_evidence = admitted.resource_attempt.evidence().clone();
        let execution_snapshot = workspace.snapshot_identity();
        let conditional = match admitted.evaluate_conditionals(
            workspace,
            &execution_snapshot,
            &resource_evidence,
            &mut counters,
        ) {
            Ok(conditional) => conditional,
            Err(stop) => return Err(conditional_stop_outcome(admitted, counters, stop)),
        };
        let WorthQueryAdmittedDirectOperation {
            bound,
            input,
            executor,
            resource_attempt,
            phase_proof,
        } = admitted;
        let resources = resource_attempt.resources().clone();
        let managed = match workspace.admit_managed_direct_run(
            bound.execution_authority(),
            bound.product(),
            resource_attempt,
        ) {
            Ok(managed) => managed,
            Err(failure) => {
                let detail = failure.detail().to_owned();
                let _ = failure.release();
                return Err(TransitionOutcome::Denied(
                    WorthQueryBoundExecutionDenial::new(
                        WorthQueryBoundExecutionDenialKind::GraphProvider,
                        detail,
                        counters,
                    ),
                ));
            }
        };
        Ok(Self {
            bound,
            input,
            executor,
            phase_proof,
            running: Some(managed.start()),
            resources,
            execution_snapshot,
            conditional,
            resource_evidence,
            counters,
        })
    }

    fn invoke_graphs(
        mut self,
    ) -> Result<
        WorthQueryGraphCompletedDirectExecution<D, O, F, L>,
        super::super::WorthQueryBoundExecutionOutcome<D, O, F, L, O::Output>,
    > {
        let graph_receipts = super::super::super::bound_graph_execution::invoke_bound_graphs(
            &self.bound,
            self.running
                .take()
                .expect("prepared direct execution owns its managed run"),
            &mut self.counters,
        )
        .map_err(TransitionOutcome::Denied)?;
        self.running = Some(graph_receipts.0);
        Ok(WorthQueryGraphCompletedDirectExecution {
            prepared: self,
            graph_receipts: graph_receipts.1,
        })
    }
}

impl<D: 'static, O, F: 'static, L: BasisOperationLane>
    WorthQueryGraphCompletedDirectExecution<D, O, F, L>
where
    O: WorthQueryExecutableDomainOperation<
        D,
        F,
        Execution = super::super::super::WorthQueryDirectOperation,
    >,
{
    fn invoke_executor(
        self,
        workspace: &mut crate::runtime::WorthQueryWorkspace,
    ) -> super::super::WorthQueryBoundExecutionOutcome<D, O, F, L, O::Output> {
        let Self {
            prepared,
            graph_receipts,
        } = self;
        let WorthQueryPreparedDirectExecution {
            bound,
            input,
            executor,
            phase_proof,
            running,
            resources,
            execution_snapshot,
            conditional,
            resource_evidence,
            mut counters,
        } = prepared;
        let running = running.expect("completed graph execution owns its managed run");
        let context = WorthQueryOperationExecutionContext::new(
            bound.definition(),
            bound.binding_identity(),
            bound.basis().capability_digest(),
            bound.basis().normalized(),
            executor.installed_read.as_ref(),
            &graph_receipts,
            &resources,
            running.provider_session_identity(),
        );
        counters.executor_contacts += 1;
        let (material, primary_read_contacts) =
            match executor.execute::<D, O, F>(input, &context, workspace) {
                Ok(material) => material,
                Err(failure) => {
                    let kind = classify_executor_failure(&bound, failure.class().clone());
                    let detail = failure.detail().to_owned();
                    let cleanup = running.abandon().cleanup();
                    return TransitionOutcome::Failed(
                        WorthQueryBoundExecutionDenial::new(kind, detail, counters)
                            .with_graph_receipts(graph_receipts)
                            .with_managed_cleanup(cleanup),
                    );
                }
            };
        counters.primary_read_contacts += primary_read_contacts;
        let (output, result_state, warnings, material) = material.into_parts();
        WorthQueryExecutorCompletedDirectExecution {
            bound,
            phase_proof,
            running,
            resources,
            output,
            result_state,
            warnings,
            material,
            graph_receipts,
            snapshot: execution_snapshot,
            conditional,
            resource_evidence,
            counters,
        }
        .validate_terminal()
    }
}

impl<D: 'static, O, F: 'static, L: BasisOperationLane, Output>
    WorthQueryExecutorCompletedDirectExecution<D, O, F, L, Output>
where
    O: WorthQueryExecutableDomainOperation<D, F, Output = Output>,
    Output: super::super::super::WorthQueryOperationOutput,
{
    fn validate_terminal(
        mut self,
    ) -> super::super::WorthQueryBoundExecutionOutcome<D, O, F, L, Output> {
        self.counters.terminal_posture_checks += 1;
        if !self
            .bound
            .definition()
            .semantics()
            .terminal
            .result_states
            .contains(&self.result_state)
        {
            let cleanup = self.running.abandon().cleanup();
            return TransitionOutcome::Denied(
                WorthQueryBoundExecutionDenial::new(
                    WorthQueryBoundExecutionDenialKind::UndeclaredResultState,
                    "executor returned a result state absent from the installed terminal contract",
                    self.counters,
                )
                .with_graph_receipts(self.graph_receipts)
                .with_managed_cleanup(cleanup),
            );
        }
        WorthQueryValidatedDirectEvidenceCompletion {
            bound: self.bound,
            phase_proof: self.phase_proof,
            running: Some(self.running),
            resources: self.resources,
            output: self.output,
            result_state: self.result_state,
            warnings: self.warnings,
            material: self.material,
            graph_receipts: self.graph_receipts,
            snapshot: self.snapshot,
            conditional: self.conditional,
            resource_evidence: self.resource_evidence,
            counters: self.counters,
        }
        .finish()
    }
}

impl<D, O, F, L: BasisOperationLane> WorthQueryAdmittedDirectOperation<D, O, F, L>
where
    O: WorthQueryExecutableDomainOperation<D, F>,
{
    fn evaluate_conditionals(
        &self,
        workspace: &mut crate::runtime::WorthQueryWorkspace,
        execution_snapshot: &crate::memory_workspace::WorthQuerySnapshotIdentity,
        resource_evidence: &super::super::super::WorthQueryExecutionResourceAttemptEvidence,
        counters: &mut WorthQueryOperationExecutionCounters,
    ) -> Result<
        Vec<crate::domain_installation::WorthQueryConditionalProvenance>,
        crate::domain_installation::WorthQueryConditionalEvaluationStop,
    > {
        let execution_identity = format!(
            "{}:bound-capability:{}",
            self.bound.binding_identity(),
            self.bound.capability_identity()
        );
        crate::domain_installation::evaluate_bound_conditionals(
            &self.bound,
            crate::domain_installation::WorthQueryConditionalEvaluationPass {
                workspace,
                snapshot: execution_snapshot,
                execution_identity: &execution_identity,
                scope: crate::domain_installation::WorthQueryConditionalEvaluationScope::Operation,
                workflow_run_identity: None,
                attempt: 1,
                resources: self.resource_attempt.resources(),
                resource_evidence,
                counters,
            },
        )
    }
}

fn conditional_stop_outcome<D, O, F, L: BasisOperationLane>(
    admitted: WorthQueryAdmittedDirectOperation<D, O, F, L>,
    counters: WorthQueryOperationExecutionCounters,
    stop: crate::domain_installation::WorthQueryConditionalEvaluationStop,
) -> super::super::WorthQueryBoundExecutionOutcome<D, O, F, L, O::Output>
where
    O: WorthQueryExecutableDomainOperation<D, F>,
{
    match stop {
        crate::domain_installation::WorthQueryConditionalEvaluationStop::Deferred(conditional) => {
            TransitionOutcome::Deferred(
                crate::domain_installation::WorthQueryDeferredDomainOperation {
                    admitted,
                    conditional,
                    counters,
                },
            )
        }
        crate::domain_installation::WorthQueryConditionalEvaluationStop::Failed {
            kind,
            detail,
        } => TransitionOutcome::Failed(WorthQueryBoundExecutionDenial::new(
            WorthQueryBoundExecutionDenialKind::ConditionalExecution(kind),
            detail,
            counters,
        )),
        crate::domain_installation::WorthQueryConditionalEvaluationStop::Reentry(denial) => {
            TransitionOutcome::Denied(WorthQueryBoundExecutionDenial::new(
                WorthQueryBoundExecutionDenialKind::ConditionalReentry(denial),
                "Signal decision did not re-enter the exact bound Query operation",
                counters,
            ))
        }
    }
}

fn classify_executor_failure<D, O, F, L: BasisOperationLane>(
    bound: &crate::domain_installation::WorthQueryBoundDomainOperation<D, O, F, L>,
    class: crate::domain_installation::WorthQueryOperationFailureClass,
) -> WorthQueryBoundExecutionDenialKind {
    if bound
        .definition()
        .semantics()
        .terminal
        .failure_classes
        .contains(&class)
    {
        WorthQueryBoundExecutionDenialKind::Executor(class)
    } else {
        WorthQueryBoundExecutionDenialKind::UndeclaredFailureClass(class)
    }
}
