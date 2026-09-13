use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use crate::basis_lifecycle::BasisOperationLane;

use super::{
    WorthQueryAdmittedWorkflowOperation, WorthQueryAdmittedWorkflowResourcePlan,
    WorthQueryExecutionResourceAttemptEvidence, WorthQueryInstalledWorkflowGraph,
    WorthQueryInstalledWorkflowStageExecutor, WorthQueryWorkflowRunCounters,
    WorthQueryWorkflowStageReceipt, WorthQueryWorkflowStartDenial,
    WorthQueryWorkflowStartDenialKind,
};
use super::{WorthQueryExecutableDomainOperation, WorthQueryWorkflowOperation};
use crate::domain_installation::operation_authority_chain::{
    mint_operation_phase_proof, operation_phase_basis, WorthQueryOperationPhaseProof,
    WorthQueryWorkflowRunPhase,
};
use crate::domain_installation::WorthQueryBoundDomainOperation;
use worth_proof::TransitionOutcome;

pub type WorthQueryWorkflowStartOutcome<D, O, F, L> = TransitionOutcome<
    WorthQueryWorkflowRun<D, O, F, L>,
    WorthQueryWorkflowStartDenial,
    crate::domain_installation::WorthQueryDeferredWorkflowStart<D, O, F, L>,
    WorthQueryWorkflowStartDenial,
    WorthQueryWorkflowStartDenial,
    WorthQueryWorkflowStartDenial,
>;

#[derive(Debug)]
pub(super) struct WorthQueryWorkflowRunAuthorityProof {
    pub(super) domain_authority:
        Arc<crate::domain_installation::WorthQueryInstalledDomainAuthority>,
    pub(super) proof: WorthQueryOperationPhaseProof<WorthQueryWorkflowRunPhase>,
}

impl WorthQueryWorkflowRunAuthorityProof {
    pub(super) fn operation_identity(&self) -> &str {
        &operation_phase_basis(&self.proof).operation_identity
    }

    pub(super) fn binding_identity(&self) -> &str {
        &operation_phase_basis(&self.proof).binding_identity
    }

    pub(super) fn basis_identity(&self) -> &str {
        &operation_phase_basis(&self.proof).basis_identity
    }

    pub(super) fn capability_identity(&self) -> u64 {
        operation_phase_basis(&self.proof).capability_identity
    }
}

pub struct WorthQueryWorkflowRun<D, O, F, L: BasisOperationLane> {
    pub(super) bound: WorthQueryBoundDomainOperation<D, O, F, L>,
    pub(super) graph: Arc<WorthQueryInstalledWorkflowGraph>,
    pub(super) executor: Arc<WorthQueryInstalledWorkflowStageExecutor>,
    pub(super) identity: String,
    pub(super) completed: BTreeSet<String>,
    pub(super) receipt_index: BTreeMap<String, usize>,
    pub(super) receipts: Vec<WorthQueryWorkflowStageReceipt>,
    pub(super) counters: WorthQueryWorkflowRunCounters,
    pub(super) parallel_posture:
        crate::domain_installation::operating_world::WorthQueryBoundWorkflowParallelPosture,
    pub(super) active_parallel_admission:
        Option<Arc<super::WorthQueryWorkflowParallelAdmissionReceipt>>,
    pub(super) authority_proof: Arc<WorthQueryWorkflowRunAuthorityProof>,
    pub(super) operation_conditional:
        Vec<crate::domain_installation::WorthQueryConditionalProvenance>,
    pub(super) artifact_registry:
        Arc<crate::domain_installation::WorthQueryWorkflowArtifactRegistry>,
    pub(super) _artifact_registry_guard: WorthQueryWorkflowArtifactRegistryGuard,
    pub(super) domain_evidence_ledger: super::WorthQueryDomainEvidenceAdmissionLedger,
    pub(super) managed:
        Option<worth_query_execution::facade::runtime::WorthQueryRunningWorkflowRun>,
    pub(super) managed_cleanup:
        Option<worth_query_execution::facade::runtime::WorthQueryWorkflowRunCleanupReceipt>,
    pub(super) resources: WorthQueryAdmittedWorkflowResourcePlan,
    pub(super) operation_resource_evidence: WorthQueryExecutionResourceAttemptEvidence,
}

pub(super) struct WorthQueryWorkflowArtifactRegistryGuard(
    Arc<crate::domain_installation::WorthQueryWorkflowArtifactRegistry>,
);

impl Drop for WorthQueryWorkflowArtifactRegistryGuard {
    fn drop(&mut self) {
        self.0.close_cancelled();
    }
}

impl<D: 'static, O: 'static, F: 'static, L: BasisOperationLane>
    WorthQueryAdmittedWorkflowOperation<D, O, F, L>
where
    O: WorthQueryExecutableDomainOperation<D, F, Execution = WorthQueryWorkflowOperation>,
{
    pub fn start_workflow(
        self,
        workspace: &mut crate::runtime::WorthQueryWorkspace,
    ) -> WorthQueryWorkflowStartOutcome<D, O, F, L> {
        self.start_workflow_attempt(workspace, 1)
    }

    pub(super) fn start_workflow_attempt(
        self,
        workspace: &mut crate::runtime::WorthQueryWorkspace,
        attempt: u64,
    ) -> WorthQueryWorkflowStartOutcome<D, O, F, L> {
        let mut counters = WorthQueryWorkflowRunCounters::default();
        let operation_resource_evidence = self.resource_attempt.evidence().clone();
        let snapshot = workspace.snapshot_identity();
        let artifact_authority = match self.resource_attempt.begin_managed_workflow_artifacts() {
            Ok(authority) => authority,
            Err(denial) => {
                return TransitionOutcome::Denied(WorthQueryWorkflowStartDenial::new(
                    WorthQueryWorkflowStartDenialKind::ArtifactAuthority(denial),
                    counters,
                ));
            }
        };
        let identity = artifact_authority.run_identity().to_owned();
        let operation_conditional =
            match super::workflow_conditional_start_evaluation::evaluate(
                &self.bound,
                super::workflow_conditional_start_evaluation::ConditionalWorkflowStartEvaluationPass {
                    workspace,
                    snapshot: &snapshot,
                    run_identity: &identity,
                    attempt,
                    resources: self.resource_attempt.operation_resources(),
                    resource_evidence: &operation_resource_evidence,
                    run_counters: &mut counters,
                },
            ) {
                Ok(conditional) => conditional,
                Err(super::workflow_conditional_start_evaluation::ConditionalWorkflowStartStop::Deferred(conditional)) => {
                    artifact_authority.registry().close_cancelled();
                    return TransitionOutcome::Deferred(
                        crate::domain_installation::WorthQueryDeferredWorkflowStart {
                            admitted: self,
                            conditional,
                            counters,
                            run_identity: identity,
                            attempt,
                        },
                    );
                }
                Err(super::workflow_conditional_start_evaluation::ConditionalWorkflowStartStop::Denied(kind)) => {
                    artifact_authority.registry().close_cancelled();
                    let failed = matches!(
                        kind,
                        WorthQueryWorkflowStartDenialKind::ConditionalExecution(_)
                    );
                    let denial = WorthQueryWorkflowStartDenial::new(kind, counters);
                    return if failed {
                        TransitionOutcome::Failed(denial)
                    } else {
                        TransitionOutcome::Denied(denial)
                    };
                }
            };
        let proof = mint_operation_phase_proof(
            identity,
            Some(self.phase_proof.payload().identity()),
            operation_phase_basis(&self.phase_proof).clone(),
        );
        let authority_proof = Arc::new(WorthQueryWorkflowRunAuthorityProof {
            domain_authority: Arc::clone(self.bound.operation().domain_authority()),
            proof,
        });
        let WorthQueryAdmittedWorkflowOperation {
            bound,
            graph,
            executor,
            parallel_posture,
            resource_attempt,
            phase_proof: _,
        } = self;
        let resources = resource_attempt.resources().clone();
        let managed = match workspace.admit_managed_workflow_run(
            bound.execution_authority(),
            bound.product(),
            resource_attempt,
        ) {
            Ok(managed) => managed,
            Err(failure) => {
                let detail = failure.detail().to_owned();
                artifact_authority.registry().close_cancelled();
                let _ = failure.release();
                return TransitionOutcome::Denied(WorthQueryWorkflowStartDenial::new(
                    WorthQueryWorkflowStartDenialKind::ManagedRun(detail),
                    counters,
                ));
            }
        };
        let managed = match managed.start_with_artifacts(artifact_authority) {
            Ok(running) => running,
            Err(rejection) => {
                let detail = rejection.denial().detail().to_owned();
                let _ = rejection.release();
                return TransitionOutcome::Denied(WorthQueryWorkflowStartDenial::new(
                    WorthQueryWorkflowStartDenialKind::ManagedRun(detail),
                    counters,
                ));
            }
        };
        let identity = managed.artifacts().run_identity().to_owned();
        let artifact_registry = managed.artifacts().registry();
        let artifact_registry_guard =
            WorthQueryWorkflowArtifactRegistryGuard(Arc::clone(&artifact_registry));
        TransitionOutcome::Success(WorthQueryWorkflowRun {
            bound,
            graph,
            executor,
            identity: identity.clone(),
            completed: BTreeSet::new(),
            receipt_index: BTreeMap::new(),
            receipts: Vec::new(),
            counters,
            parallel_posture,
            active_parallel_admission: None,
            authority_proof,
            operation_conditional,
            artifact_registry,
            _artifact_registry_guard: artifact_registry_guard,
            domain_evidence_ledger: super::WorthQueryDomainEvidenceAdmissionLedger::default(),
            managed: Some(managed),
            managed_cleanup: None,
            resources,
            operation_resource_evidence,
        })
    }
}

impl<D, O, F, L: BasisOperationLane> WorthQueryWorkflowRun<D, O, F, L> {
    pub fn identity(&self) -> &str {
        &self.identity
    }
    pub fn receipts(&self) -> &[WorthQueryWorkflowStageReceipt] {
        &self.receipts
    }
    pub fn counters(&self) -> WorthQueryWorkflowRunCounters {
        self.counters
    }
    pub fn installed_graph(&self) -> &WorthQueryInstalledWorkflowGraph {
        &self.graph
    }
    pub fn operation_conditional_provenance(
        &self,
    ) -> &[crate::domain_installation::WorthQueryConditionalProvenance] {
        &self.operation_conditional
    }
    pub fn resources(&self) -> &WorthQueryAdmittedWorkflowResourcePlan {
        &self.resources
    }
    pub fn provider_session_identity(&self) -> &str {
        if let Some(managed) = self.managed.as_ref() {
            managed.provider_session_identity()
        } else {
            self.managed_cleanup
                .as_ref()
                .expect("terminal workflow owns its managed cleanup receipt")
                .inspection()
                .provider_session_identity()
        }
    }
    pub fn operation_resource_evidence(&self) -> &WorthQueryExecutionResourceAttemptEvidence {
        &self.operation_resource_evidence
    }

    pub(super) fn managed_run(
        &self,
    ) -> &worth_query_execution::facade::runtime::WorthQueryRunningWorkflowRun {
        self.managed
            .as_ref()
            .expect("live workflow owns its managed run")
    }
}

impl<D: 'static, O: 'static, F: 'static, L: BasisOperationLane>
    crate::domain_installation::WorthQueryDeferredWorkflowStart<D, O, F, L>
where
    O: WorthQueryExecutableDomainOperation<D, F, Execution = WorthQueryWorkflowOperation>,
{
    pub fn retry(
        self,
        workspace: &mut crate::runtime::WorthQueryWorkspace,
    ) -> WorthQueryWorkflowStartOutcome<D, O, F, L> {
        self.admitted
            .start_workflow_attempt(workspace, self.attempt + 1)
    }
}
