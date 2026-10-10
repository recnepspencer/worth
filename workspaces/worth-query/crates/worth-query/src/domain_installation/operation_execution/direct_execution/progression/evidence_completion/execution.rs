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
        let owner = workspace.advancement_owner();
        owner
            .with_advancement(|phase| {
                let execution = &phase;
                let prepared =
                    match WorthQueryPreparedDirectExecution::prepare(execution, self, workspace) {
                        Ok(prepared) => prepared,
                        Err(outcome) => return outcome,
                    };
                match prepared.invoke_graphs(execution) {
                    Ok(completed) => completed.invoke_executor(execution, workspace),
                    Err(outcome) => outcome,
                }
            })
            .unwrap_or_else(|cause| {
                TransitionOutcome::Denied(WorthQueryBoundExecutionDenial::new(
                    WorthQueryBoundExecutionDenialKind::ExecutionRequest(cause),
                    "request admission",
                    Default::default(),
                ))
            })
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
        execution: &worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase<'_>,
        admitted: WorthQueryAdmittedDirectOperation<D, O, F, L>,
        workspace: &mut crate::runtime::WorthQueryWorkspace,
    ) -> Result<Self, super::super::WorthQueryBoundExecutionOutcome<D, O, F, L, O::Output>> {
        let resource_request = execution
            .execution_request_for(&workspace.advancement_owner())
            .expect("direct admission uses its workspace advancement");
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
            execution,
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
            resource_request,
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
        execution: &worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase<'_>,
    ) -> Result<
        WorthQueryGraphCompletedDirectExecution<D, O, F, L>,
        super::super::WorthQueryBoundExecutionOutcome<D, O, F, L, O::Output>,
    > {
        let graph_receipts = super::super::super::bound_graph_execution::invoke_bound_graphs(
            execution,
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

impl<D, O, F, L: BasisOperationLane> WorthQueryAdmittedDirectOperation<D, O, F, L>
where
    O: WorthQueryExecutableDomainOperation<D, F>,
{
    fn evaluate_conditionals(
        &self,
        execution: &worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase<'_>,
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
                execution: execution
                    .execution_request_for(&workspace.advancement_owner())
                    .expect("conditional evaluation uses its workspace phase"),
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

mod executor_progression;
