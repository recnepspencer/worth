use super::*;
use std::sync::Mutex;

struct CheckpointContinuityProvider;

struct CheckpointContinuityExecution {
    step_ordinal: usize,
}

impl WorthQueryGraphProviderExecution for CheckpointContinuityExecution {
    fn advance(
        &mut self,
        step: &mut WorthQueryGraphProviderStep,
    ) -> Result<WorthQueryGraphProviderStepDisposition, WorthQueryGraphProviderFailure> {
        step.perform_work_unit(|| Ok(()))?;
        let disposition = if self.step_ordinal == 0 {
            step.record_checkpoint_available().map_err(step_failure)?;
            WorthQueryGraphProviderStepDisposition::continue_work()
        } else {
            WorthQueryGraphProviderStepDisposition::complete("checkpoint-continuity")
                .map_err(WorthQueryGraphProviderFailure::new)?
        };
        self.step_ordinal = self.step_ordinal.saturating_add(1);
        Ok(disposition)
    }

    fn dispose(&mut self) -> Result<(), WorthQueryGraphProviderFailure> {
        Ok(())
    }
}

impl WorthQueryGraphParticipationProvider<ManagedGraph> for CheckpointContinuityProvider {
    type Execution = CheckpointContinuityExecution;

    fn execution_resource_support(
        &self,
    ) -> worth_query_admission::facade::resource_admission::WorthQueryExecutionResourceSupport {
        crate::domain_computation::provider_session::execution_resource_support(
            "checkpoint-continuity-provider",
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
        admit_provider_execution(start, CheckpointContinuityExecution { step_ordinal: 0 })
    }
}

struct PreexistingArtifactResource(Arc<AtomicUsize>);

impl WorthQueryArtifactProviderResource for PreexistingArtifactResource {
    const PROVIDER_FAMILY: &'static str = "WORTH.tests.affinity.provider";

    fn canonical_semantic_projection(&self) -> Vec<u8> {
        b"preexisting-artifact".to_vec()
    }

    fn retained_bytes(&self) -> usize {
        37
    }

    fn dispose(&mut self) {
        self.0.fetch_add(1, Ordering::AcqRel);
    }
}

struct MultiCallArtifactProvider {
    begin_ordinal: Arc<AtomicUsize>,
    retained: Arc<Mutex<Option<crate::domain_computation::WorthQueryMoveOnlyArtifactHandle>>>,
    disposed: Arc<AtomicUsize>,
}

struct MultiCallArtifactExecution {
    produce: bool,
    retained: Arc<Mutex<Option<crate::domain_computation::WorthQueryMoveOnlyArtifactHandle>>>,
    disposed: Arc<AtomicUsize>,
}

impl WorthQueryGraphProviderExecution for MultiCallArtifactExecution {
    fn advance(
        &mut self,
        step: &mut WorthQueryGraphProviderStep,
    ) -> Result<WorthQueryGraphProviderStepDisposition, WorthQueryGraphProviderFailure> {
        step.perform_work_unit(|| Ok(()))?;
        if self.produce {
            let artifact = step
                .produce_artifact(
                    WorthQueryArtifactProductionEvidence::new(
                        "multi-call-provider",
                        "multi-call-dependency",
                    ),
                    PreexistingArtifactResource(Arc::clone(&self.disposed)),
                )
                .map_err(step_failure)?;
            *self
                .retained
                .lock()
                .expect("retained artifact lock should remain available") = Some(artifact);
        }
        WorthQueryGraphProviderStepDisposition::complete("multi-call-provider")
            .map_err(WorthQueryGraphProviderFailure::new)
    }

    fn dispose(&mut self) -> Result<(), WorthQueryGraphProviderFailure> {
        Ok(())
    }
}

impl WorthQueryGraphParticipationProvider<ManagedGraph> for MultiCallArtifactProvider {
    type Execution = MultiCallArtifactExecution;

    fn execution_resource_support(
        &self,
    ) -> worth_query_admission::facade::resource_admission::WorthQueryExecutionResourceSupport {
        crate::domain_computation::provider_session::execution_resource_support(
            "multi-call-artifact-provider",
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
            MultiCallArtifactExecution {
                produce: self.begin_ordinal.fetch_add(1, Ordering::AcqRel) == 0,
                retained: Arc::clone(&self.retained),
                disposed: Arc::clone(&self.disposed),
            },
        )
    }
}

#[test]
fn provider_step_evidence_ignores_preexisting_artifacts_and_keeps_checkpoint_continuity() {
    crate::domain_computation::primary_graph::with_test_advancement(|active_phase| {
        let bootstrap = active_phase.bootstrap_for_test();
        let resource_request = bootstrap.execution_request();

        let phase = &active_phase;
        let execution = phase;

        let installer = WorthQueryExecutionRuntimeInstaller::new();
        let provider_anchor = Arc::new(
        crate::domain_computation::provider_session::graph_provider::bounded_step::provider_anchor::WorthQueryGraphProviderAnchor::install::<ManagedGraph, _>(
            CheckpointContinuityProvider,
        ),
    );
        let provider_support = provider_anchor.resource_support().clone();
        let graph = super::workflow_provider_steps::installed_graph(
            &installer,
            "checkpoint-continuity-graph",
            provider_anchor,
        );
        let runtime =
            super::workflow_provider_steps::installed_runtime(installer, "checkpoint continuity");
        let operation_resources = admitted_plan("checkpoint-continuity", 8);
        let stage_resources = admitted_plan_with_graph_support(
            "checkpoint-continuity:producer",
            8,
            graph.role(),
            provider_support,
        );
        let resources = WorthQueryAdmittedWorkflowResourcePlan::assemble(
            operation_resources,
            BTreeMap::from([("producer".to_owned(), stage_resources)]),
        );
        let output =
            crate::domain_computation::artifact_owner::installed_artifact_contract_for_managed_run(
            );
        let operation = workflow_authority_with_stage_graph_and_output_artifact(
            &runtime,
            &resources,
            "producer",
            &graph,
            WorthQueryOperationGraphAccess::Observe,
            output,
        );
        let running = super::workflow_provider_steps::admitted_workflow(
            &runtime,
            &operation,
            resources,
            resource_request,
        );
        let production = running
            .artifacts()
            .production_authority("producer")
            .expect("producer artifact authority should resolve")
            .expect("producer output contract should be installed");
        let disposed = Arc::new(AtomicUsize::new(0));
        let admission = crate::domain_computation::WorthQueryArtifactProductionAuthority::admit(
            &production,
            WorthQueryArtifactProductionEvidence::new("preexisting", "preexisting-dependency"),
        );
        let preexisting =
            crate::domain_computation::WorthQueryArtifactProductionAuthority::register_exact(
                &production,
                admission,
                PreexistingArtifactResource(Arc::clone(&disposed)),
            )
            .expect("preexisting workflow artifact should register");

        let active = running
            .begin_stage_graph_execution(
                execution,
                "producer",
                &graph,
                WorthQueryManagedGraphCallRequest::new(
                    WorthQueryGraphProviderCallKind::Observe,
                    "checkpoint-continuity",
                ),
            )
            .expect("checkpoint provider should start");
        let active = match active.advance(execution) {
            WorthQueryWorkflowGraphStepOutcome::Continue(active) => active,
            _ => panic!("checkpoint-producing first step did not continue"),
        };
        let completion = match active.advance(execution) {
            WorthQueryWorkflowGraphStepOutcome::Completed(completion) => completion,
            _ => panic!("checkpoint continuity provider did not complete"),
        };
        let receipt = completion.receipt().work_report();
        assert_eq!(receipt.produced_artifact_count(), 0);
        assert_eq!(receipt.retained_artifact_count(), 0);
        assert_eq!(receipt.disposed_artifact_count(), 0);

        let terminal = completion.into_running().completed().unwrap();
        assert!(!terminal.provider_work().checkpoint_available());
        assert_eq!(
            terminal
                .provider_work()
                .checkpoint_available_observation_count(),
            1
        );
        assert_eq!(terminal.provider_work().produced_artifact_count(), 0);
        assert_eq!(terminal.provider_work().retained_artifact_count(), 0);
        assert_eq!(terminal.provider_work().retained_bytes(), 0);
        assert_eq!(terminal.artifact_evidence().retained_artifact_count(), 1);
        drop(preexisting);
        assert_eq!(disposed.load(Ordering::Acquire), 1);
        match terminal.cleanup() {
            WorthQueryWorkflowRunCleanupOutcome::Complete(_) => {}
            _ => panic!("released preexisting artifact should permit exact cleanup"),
        }
    });
}

#[test]
fn governed_artifact_retention_survives_a_later_provider_call() {
    crate::domain_computation::primary_graph::with_test_advancement(|active_phase| {
        let bootstrap = active_phase.bootstrap_for_test();
        let resource_request = bootstrap.execution_request();

        let phase = &active_phase;
        let execution = phase;

        let installer = WorthQueryExecutionRuntimeInstaller::new();
        let retained = Arc::new(Mutex::new(None));
        let disposed = Arc::new(AtomicUsize::new(0));
        let provider_anchor = Arc::new(
        crate::domain_computation::provider_session::graph_provider::bounded_step::provider_anchor::WorthQueryGraphProviderAnchor::install::<ManagedGraph, _>(
            MultiCallArtifactProvider {
                begin_ordinal: Arc::new(AtomicUsize::new(0)),
                retained: Arc::clone(&retained),
                disposed: Arc::clone(&disposed),
            },
        ),
    );
        let provider_support = provider_anchor.resource_support().clone();
        let graph = super::workflow_provider_steps::installed_graph(
            &installer,
            "multi-call-artifact-graph",
            provider_anchor,
        );
        let runtime = super::workflow_provider_steps::installed_runtime(installer, "multi-call");
        let operation_resources = admitted_plan("multi-call-artifact", 8);
        let stage_resources = admitted_plan_with_graph_support(
            "multi-call-artifact:producer",
            8,
            graph.role(),
            provider_support,
        );
        let resources = WorthQueryAdmittedWorkflowResourcePlan::assemble(
            operation_resources,
            BTreeMap::from([("producer".to_owned(), stage_resources)]),
        );
        let output =
            crate::domain_computation::artifact_owner::installed_artifact_contract_for_managed_run(
            );
        let operation = workflow_authority_with_stage_graph_and_output_artifact(
            &runtime,
            &resources,
            "producer",
            &graph,
            WorthQueryOperationGraphAccess::Observe,
            output,
        );
        let running = super::workflow_provider_steps::admitted_workflow(
            &runtime,
            &operation,
            resources,
            resource_request,
        );

        let first = complete_observe_call(execution, running, &graph, "multi-call-first");
        assert_eq!(first.receipt().work_report().produced_artifact_count(), 1);
        assert_eq!(first.receipt().work_report().retained_artifact_count(), 1);
        let second =
            complete_observe_call(execution, first.into_running(), &graph, "multi-call-second");
        assert_eq!(second.receipt().work_report().produced_artifact_count(), 0);
        assert_eq!(second.receipt().work_report().retained_artifact_count(), 0);

        let terminal = second.into_running().completed().unwrap();
        assert_eq!(terminal.provider_work().produced_artifact_count(), 1);
        assert_eq!(terminal.provider_work().retained_artifact_count(), 1);
        assert_eq!(terminal.provider_work().disposed_artifact_count(), 0);
        assert_eq!(terminal.provider_work().retained_bytes(), 37);
        assert_eq!(terminal.artifact_evidence().retained_artifact_count(), 1);

        drop(
            retained
                .lock()
                .expect("retained artifact lock should remain available")
                .take(),
        );
        assert_eq!(disposed.load(Ordering::Acquire), 1);
        match terminal.cleanup() {
            WorthQueryWorkflowRunCleanupOutcome::Complete(receipt) => {
                assert_eq!(
                    receipt
                        .inspection()
                        .artifact_evidence()
                        .disposed_artifact_count(),
                    1
                );
            }
            _ => panic!("released governed artifact should permit cleanup"),
        }
    });
}

fn complete_observe_call(
    execution: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,

    running: crate::domain_computation::WorthQueryRunningWorkflowRun,
    graph: &WorthQueryInstalledGraphParticipationAuthority,
    scope: &str,
) -> crate::domain_computation::WorthQueryCompletedWorkflowGraphExecution {
    let active = running
        .begin_stage_graph_execution(
            execution,
            "producer",
            graph,
            WorthQueryManagedGraphCallRequest::new(WorthQueryGraphProviderCallKind::Observe, scope),
        )
        .expect("multi-call provider should start");
    match active.advance(execution) {
        WorthQueryWorkflowGraphStepOutcome::Completed(completion) => completion,
        _ => panic!("multi-call provider did not complete"),
    }
}

fn step_failure(
    denial: crate::domain_computation::WorthQueryGraphProviderStepDenial,
) -> WorthQueryGraphProviderFailure {
    WorthQueryGraphProviderFailure::new(denial.detail())
}
