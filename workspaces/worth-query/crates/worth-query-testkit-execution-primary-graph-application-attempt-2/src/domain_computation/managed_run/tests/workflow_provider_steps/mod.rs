use crate::domain_computation::managed_run::tests::admit_provider_execution;
use crate::domain_computation::managed_run::tests::graph_material;
use crate::domain_computation::managed_run::tests::ManagedGraph;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use worth_query_admission::facade::resource_admission::WorthQueryAdmittedWorkflowResourcePlan;
use worth_query_execution::facade::domain_computation::WorthQueryArtifactProductionEvidence;
use worth_query_execution::facade::domain_computation::WorthQueryCooperativeGraphProviderExecution;
use worth_query_execution::facade::domain_computation::WorthQueryExecutionRuntime;
use worth_query_execution::facade::domain_computation::WorthQueryExecutionRuntimeInstaller;
use worth_query_execution::facade::domain_computation::WorthQueryGraphProviderCall;
use worth_query_execution::facade::domain_computation::WorthQueryGraphProviderExecutionStart;
use worth_query_execution::facade::domain_computation::WorthQueryGraphProviderFailure;
use worth_query_execution::facade::domain_computation::WorthQueryGraphProviderStep;
use worth_query_execution::facade::domain_computation::WorthQueryGraphProviderStepDisposition;
use worth_query_execution::facade::domain_computation::WorthQueryWorkflowRunCleanupOutcome;
use worth_query_installation::facade::WorthQueryInstalledGraphParticipationAuthority;

use crate::domain_computation::managed_run::tests::*;
#[derive(Clone)]
pub enum WorkflowStageBehavior {
    Project,
    Fail,
    ArtifactCheckpoint(Arc<AtomicUsize>),
}
pub struct WorkflowStageProvider {
    pub behavior: WorkflowStageBehavior,
}
pub struct WorkflowStageExecution {
    pub behavior: WorkflowStageBehavior,
    pub artifact:
        Option<worth_query_execution::facade::domain_computation::WorthQueryMoveOnlyArtifactHandle>,
}
impl WorthQueryGraphProviderExecution for WorkflowStageExecution {
    fn advance(
        &mut self,
        step: &mut WorthQueryGraphProviderStep,
    ) -> Result<WorthQueryGraphProviderStepDisposition, WorthQueryGraphProviderFailure> {
        match &self.behavior {
            WorkflowStageBehavior::Project => {
                for _ in 0..3 {
                    step.perform_work_unit(|| Ok(()))?;
                }
                step.emit_projection_chunk(graph_material())
                    .map_err(step_failure)?;
            }
            WorkflowStageBehavior::Fail => {
                for _ in 0..3 {
                    step.perform_work_unit(|| Ok(()))?;
                }
                return Err(WorthQueryGraphProviderFailure::new(
                    "workflow provider failed after governed work",
                ));
            }
            WorkflowStageBehavior::ArtifactCheckpoint(disposed) => {
                step.perform_work_unit(|| Ok(()))?;
                self.artifact = Some(
                    step.produce_artifact(
                        WorthQueryArtifactProductionEvidence::new(
                            "step-provenance",
                            "step-dependency",
                        ),
                        StepArtifactResource(Arc::clone(disposed)),
                    )
                    .map_err(step_failure)?,
                );
                step.record_checkpoint_available().map_err(step_failure)?;
            }
        }
        WorthQueryGraphProviderStepDisposition::complete("workflow-stage-receipt")
            .map_err(WorthQueryGraphProviderFailure::new)
    }

    fn dispose(&mut self) -> Result<(), WorthQueryGraphProviderFailure> {
        Ok(())
    }
}
impl WorthQueryGraphParticipationProvider<ManagedGraph> for WorkflowStageProvider {
    type Execution = WorkflowStageExecution;

    fn execution_resource_support(
        &self,
    ) -> worth_query_admission::facade::resource_admission::WorthQueryExecutionResourceSupport {
        crate::domain_computation::provider_session::tests::execution_resource_support(
            "workflow-stage",
            8,
        )
    }

    fn begin(
        &self,
        _call: &WorthQueryGraphProviderCall,
        start: &mut WorthQueryGraphProviderExecutionStart,
    ) -> Result<
        WorthQueryCooperativeGraphProviderExecution<Self::Execution>,
        WorthQueryGraphProviderFailure,
    > {
        admit_provider_execution(
            start,
            WorkflowStageExecution {
                behavior: self.behavior.clone(),
                artifact: None,
            },
        )
    }
}
pub struct StepArtifactResource(Arc<AtomicUsize>);
impl WorthQueryArtifactProviderResource for StepArtifactResource {
    const PROVIDER_FAMILY: &'static str = "WORTH.tests.affinity.provider";

    fn canonical_semantic_projection(&self) -> Vec<u8> {
        b"step-artifact".to_vec()
    }

    fn retained_bytes(&self) -> usize {
        1
    }

    fn dispose(&mut self) {
        self.0.fetch_add(1, Ordering::AcqRel);
    }
}
pub fn provider_anchor(
    behavior: WorkflowStageBehavior,
) -> Arc<worth_query_execution::facade::integration::WorthQueryGraphProviderAnchor> {
    Arc::new(
        worth_query_execution::facade::integration::WorthQueryGraphProviderAnchor::install::<
            ManagedGraph,
            _,
        >(WorkflowStageProvider { behavior }),
    )
}
pub fn installed_graph(
    installer: &WorthQueryExecutionRuntimeInstaller,
    identity: &str,
    provider_anchor: Arc<worth_query_execution::facade::integration::WorthQueryGraphProviderAnchor>,
) -> WorthQueryInstalledGraphParticipationAuthority {
    WorthQueryInstalledGraphParticipationAuthority::install(
        installer.installation_runtime(),
        identity,
        "workflow-bounded-provider",
        false,
        Option::<String>::None,
        provider_anchor,
    )
    .expect("workflow graph should install")
}
pub fn installed_runtime(
    installer: WorthQueryExecutionRuntimeInstaller,
    label: &str,
) -> WorthQueryExecutionRuntime {
    installer
        .install(
            worth_query_installation::facade::WorthQueryInstallationGeneration::initial(),
            std::iter::empty(),
        )
        .unwrap_or_else(|_| panic!("{label} runtime should install"))
        .into_parts()
        .0
}
pub fn admitted_workflow(
    runtime: &WorthQueryExecutionRuntime,
    operation: &worth_query_execution::facade::domain_computation::WorthQueryExecutionBoundOperationAuthority,
    resources: WorthQueryAdmittedWorkflowResourcePlan,
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) -> worth_query_execution::facade::domain_computation::WorthQueryRunningWorkflowRun {
    let attempt = runtime
        .start_workflow_resource_attempt(operation, resources)
        .expect("workflow graph resources should reserve");
    let lower = worth_query_testkit_execution_primary_graph_application_attempt::domain_computation::managed_run::tests::causal_fixture::managed_admission_context(&resource_request);
    runtime
        .managed_run_admission(&lower.bridge, &lower.relational)
        .admit_workflow(operation, attempt, lower.read_request(), resource_request)
        .expect("workflow graph run should admit")
        .start()
        .expect("workflow graph run should start")
}
pub fn workflow_cleanup(
    outcome: WorthQueryWorkflowRunCleanupOutcome,
) -> worth_query_execution::facade::domain_computation::WorthQueryWorkflowRunCleanupReceipt {
    match outcome {
        WorthQueryWorkflowRunCleanupOutcome::Complete(receipt) => receipt,
        WorthQueryWorkflowRunCleanupOutcome::Pending(_) => {
            panic!("workflow unexpectedly retained artifact owners")
        }
        WorthQueryWorkflowRunCleanupOutcome::RecoveryRequired(failure) => {
            panic!("workflow cleanup failed: {failure:?}")
        }
    }
}
pub fn assert_workflow_artifact_evidence(
    evidence: worth_query_execution::facade::domain_computation::WorthQueryWorkflowArtifactRegistryEvidence,
    expected: (usize, usize, usize, usize),
) {
    assert_eq!(
        (
            evidence.produced_artifact_count(),
            evidence.retained_artifact_count(),
            evidence.disposed_artifact_count(),
            evidence.retained_bytes(),
        ),
        expected
    );
}
pub fn step_failure(
    denial: worth_query_execution::facade::domain_computation::WorthQueryGraphProviderStepDenial,
) -> WorthQueryGraphProviderFailure {
    WorthQueryGraphProviderFailure::new(denial.detail())
}
