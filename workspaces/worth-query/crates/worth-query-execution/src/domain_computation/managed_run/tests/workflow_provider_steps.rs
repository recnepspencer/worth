use super::*;

#[derive(Clone)]
enum WorkflowStageBehavior {
    Project,
    Fail,
    ArtifactCheckpoint(Arc<AtomicUsize>),
}

struct WorkflowStageProvider {
    behavior: WorkflowStageBehavior,
}

struct WorkflowStageExecution {
    behavior: WorkflowStageBehavior,
    artifact: Option<crate::domain_computation::WorthQueryMoveOnlyArtifactHandle>,
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
        crate::domain_computation::provider_session::execution_resource_support("workflow-stage", 8)
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

struct StepArtifactResource(Arc<AtomicUsize>);

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

#[test]
fn workflow_stage_provider_call_uses_stage_resources_and_receipt_evidence() {
    let installer = WorthQueryExecutionRuntimeInstaller::new();
    let provider_anchor = provider_anchor(WorkflowStageBehavior::Project);
    let provider_support = provider_anchor.resource_support().clone();
    let graph = installed_graph(&installer, "workflow-graph", provider_anchor);
    let runtime = installed_runtime(installer, "workflow graph");
    let operation_resources = admitted_plan("workflow-graph-binding", 8);
    let stage_resources = admitted_plan_with_graph_support(
        "workflow-graph-binding:stage",
        4,
        graph.role(),
        provider_support,
    );
    let resources = WorthQueryAdmittedWorkflowResourcePlan::assemble(
        operation_resources,
        BTreeMap::from([("stage".to_owned(), stage_resources)]),
    );
    let operation = workflow_authority_with_stage_graph(
        &runtime,
        &resources,
        "stage",
        &graph,
        WorthQueryOperationGraphAccess::Project,
    );
    let running = admitted_workflow(&runtime, &operation, resources);
    let active = running
        .begin_stage_graph_execution(
            "stage",
            &graph,
            WorthQueryManagedGraphCallRequest::new(
                WorthQueryGraphProviderCallKind::Project,
                "workflow-stage-project",
            ),
        )
        .expect("installed stage resources should start the bounded provider");
    let pending = match active.advance() {
        WorthQueryWorkflowGraphStepOutcome::ChunkReady(pending) => pending,
        _ => panic!("workflow stage did not expose its bounded result chunk"),
    };
    assert_eq!(pending.queue_depth(), 1);
    let completion = match pending.acknowledge() {
        WorthQueryWorkflowGraphStepOutcome::Completed(completion) => completion,
        _ => panic!("acknowledged workflow stage did not complete"),
    };
    let stream = completion
        .receipt()
        .graph_read_stream_evidence()
        .expect("workflow managed projection should be streamed");
    assert_eq!(stream.row_count(), 1);
    assert_eq!(stream.product().rows().count(), 1);
    assert_eq!(
        completion.receipt().work_report().output_retained_bytes(),
        stream.retained_bytes()
    );
    let terminal = completion
        .into_running()
        .completed()
        .expect("step-bound stage work should complete");
    assert_eq!(terminal.provider_work().completed_work_units(), 3);
    let cleanup = workflow_cleanup(terminal.cleanup());
    assert_eq!(
        cleanup
            .inspection()
            .provider_work()
            .admitted_receipt_count(),
        1
    );
}

#[test]
fn workflow_step_derives_artifact_and_checkpoint_evidence_from_governed_ports() {
    let installer = WorthQueryExecutionRuntimeInstaller::new();
    let disposed = Arc::new(AtomicUsize::new(0));
    let provider_anchor = provider_anchor(WorkflowStageBehavior::ArtifactCheckpoint(Arc::clone(
        &disposed,
    )));
    let provider_support = provider_anchor.resource_support().clone();
    let graph = installed_graph(&installer, "workflow-artifact-graph", provider_anchor);
    let runtime = installed_runtime(installer, "workflow artifact");
    let operation_resources = admitted_plan("workflow-artifact-step", 8);
    let stage_resources = admitted_plan_with_graph_support(
        "workflow-artifact-step:producer",
        4,
        graph.role(),
        provider_support,
    );
    let resources = WorthQueryAdmittedWorkflowResourcePlan::assemble(
        operation_resources,
        BTreeMap::from([("producer".to_owned(), stage_resources)]),
    );
    let output =
        crate::domain_computation::artifact_owner::installed_artifact_contract_for_managed_run();
    let operation = workflow_authority_with_stage_graph_and_output_artifact(
        &runtime,
        &resources,
        "producer",
        &graph,
        WorthQueryOperationGraphAccess::Observe,
        output,
    );
    let running = admitted_workflow(&runtime, &operation, resources);
    let active = running
        .begin_stage_graph_execution(
            "producer",
            &graph,
            WorthQueryManagedGraphCallRequest::new(
                WorthQueryGraphProviderCallKind::Observe,
                "workflow-artifact-step",
            ),
        )
        .expect("workflow artifact provider should start");
    let completion = match active.advance() {
        WorthQueryWorkflowGraphStepOutcome::Completed(completion) => completion,
        _ => panic!("governed artifact and checkpoint step did not complete"),
    };
    let report = completion.receipt().work_report();
    assert_eq!(report.produced_artifact_count(), 1);
    assert_eq!(report.retained_artifact_count(), 1);
    assert_eq!(report.disposed_artifact_count(), 0);
    assert_eq!(report.retained_bytes(), 1);
    assert_eq!(disposed.load(Ordering::Acquire), 1);
    let terminal = completion
        .into_running()
        .completed()
        .expect("governed artifact provider work should settle");
    assert!(terminal.provider_work().checkpoint_available());
    assert_eq!(terminal.provider_work().produced_artifact_count(), 1);
    assert_eq!(terminal.provider_work().retained_artifact_count(), 0);
    assert_eq!(terminal.provider_work().disposed_artifact_count(), 1);
    assert_eq!(terminal.provider_work().retained_bytes(), 0);
    assert_workflow_artifact_evidence(terminal.artifact_evidence(), (1, 0, 1, 0));
    let cleanup = workflow_cleanup(terminal.cleanup());
    assert_eq!(
        cleanup.inspection().disposition(),
        WorthQueryManagedRunCleanupDisposition::CleanupComplete
    );
}

#[test]
fn failed_workflow_stage_preserves_governed_work_and_requires_recovery() {
    let installer = WorthQueryExecutionRuntimeInstaller::new();
    let provider_anchor = provider_anchor(WorkflowStageBehavior::Fail);
    let provider_support = provider_anchor.resource_support().clone();
    let graph = installed_graph(&installer, "uncertain-workflow-graph", provider_anchor);
    let runtime = installed_runtime(installer, "uncertain workflow");
    let operation_resources = admitted_plan("uncertain-workflow", 8);
    let stage_resources = admitted_plan_with_graph_support(
        "uncertain-workflow:stage",
        4,
        graph.role(),
        provider_support,
    );
    let resources = WorthQueryAdmittedWorkflowResourcePlan::assemble(
        operation_resources,
        BTreeMap::from([("stage".to_owned(), stage_resources)]),
    );
    let operation = workflow_authority_with_stage_graph(
        &runtime,
        &resources,
        "stage",
        &graph,
        WorthQueryOperationGraphAccess::Project,
    );
    let running = admitted_workflow(&runtime, &operation, resources);
    let active = running
        .begin_stage_graph_execution(
            "stage",
            &graph,
            WorthQueryManagedGraphCallRequest::new(
                WorthQueryGraphProviderCallKind::Project,
                "uncertain-workflow-stage",
            ),
        )
        .expect("installed stage resources should start the provider call");
    let terminal = match active.advance() {
        WorthQueryWorkflowGraphStepOutcome::Failed(terminal) => terminal,
        _ => panic!("failed workflow provider advanced the managed lane"),
    };
    assert_eq!(
        terminal.provider_work().session_disposition(),
        WorthQueryManagedProviderSessionDisposition::Uncertain
    );
    assert_eq!(terminal.provider_work().abandoned_call_count(), 1);
    assert_eq!(terminal.provider_work().completed_work_units(), 3);
    assert_eq!(
        workflow_cleanup(terminal.cleanup())
            .inspection()
            .disposition(),
        WorthQueryManagedRunCleanupDisposition::RecoveryRequired
    );
}

fn provider_anchor(
    behavior: WorkflowStageBehavior,
) -> Arc<
    crate::domain_computation::provider_session::graph_provider::bounded_step::provider_anchor::WorthQueryGraphProviderAnchor,
>{
    Arc::new(
        crate::domain_computation::provider_session::graph_provider::bounded_step::provider_anchor::WorthQueryGraphProviderAnchor::install::<ManagedGraph, _>(
            WorkflowStageProvider { behavior },
        ),
    )
}

pub(super) fn installed_graph(
    installer: &WorthQueryExecutionRuntimeInstaller,
    identity: &str,
    provider_anchor: Arc<
        crate::domain_computation::provider_session::graph_provider::bounded_step::provider_anchor::WorthQueryGraphProviderAnchor,
    >,
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

pub(super) fn installed_runtime(
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

pub(super) fn admitted_workflow(
    runtime: &WorthQueryExecutionRuntime,
    operation: &crate::domain_computation::WorthQueryExecutionBoundOperationAuthority,
    resources: WorthQueryAdmittedWorkflowResourcePlan,
) -> crate::domain_computation::WorthQueryRunningWorkflowRun {
    let attempt = runtime
        .start_workflow_resource_attempt(operation, resources)
        .expect("workflow graph resources should reserve");
    let lower = causal_fixture::managed_admission_context();
    runtime
        .managed_run_admission(&lower.bridge, &lower.relational)
        .admit_workflow(operation, attempt, lower.read_request())
        .expect("workflow graph run should admit")
        .start()
        .expect("workflow graph run should start")
}

fn workflow_cleanup(
    outcome: WorthQueryWorkflowRunCleanupOutcome,
) -> crate::domain_computation::WorthQueryWorkflowRunCleanupReceipt {
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

fn assert_workflow_artifact_evidence(
    evidence: crate::domain_computation::WorthQueryWorkflowArtifactRegistryEvidence,
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

fn step_failure(
    denial: crate::domain_computation::WorthQueryGraphProviderStepDenial,
) -> WorthQueryGraphProviderFailure {
    WorthQueryGraphProviderFailure::new(denial.detail())
}
