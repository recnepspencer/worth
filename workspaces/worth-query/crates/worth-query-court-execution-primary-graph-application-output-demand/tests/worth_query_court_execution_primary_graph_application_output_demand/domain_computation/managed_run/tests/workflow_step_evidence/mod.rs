use crate::domain_computation::managed_run::tests::*;
use std::sync::Mutex;

pub struct CheckpointContinuityProvider;

pub struct CheckpointContinuityExecution {
    pub step_ordinal: usize,
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
        worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::provider_session::tests::execution_resource_support(
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

pub struct PreexistingArtifactResource(Arc<AtomicUsize>);

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

pub struct MultiCallArtifactProvider {
    pub begin_ordinal: Arc<AtomicUsize>,
    pub retained: Arc<
        Mutex<
            Option<
                worth_query_execution::facade::domain_computation::WorthQueryMoveOnlyArtifactHandle,
            >,
        >,
    >,
    pub disposed: Arc<AtomicUsize>,
}

pub struct MultiCallArtifactExecution {
    pub produce: bool,
    pub retained: Arc<
        Mutex<
            Option<
                worth_query_execution::facade::domain_computation::WorthQueryMoveOnlyArtifactHandle,
            >,
        >,
    >,
    pub disposed: Arc<AtomicUsize>,
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
        worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::provider_session::tests::execution_resource_support(
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
    worth_query_execution::court_doors_for_test::domain_computation::primary_graph::request_execution::advancement::with_test_advancement(|active_phase| {
        let bootstrap = active_phase.bootstrap_for_test();
        let resource_request = bootstrap.execution_request();

        let phase = &active_phase;
        let execution = phase;

        let installer = WorthQueryExecutionRuntimeInstaller::new();
        let provider_anchor = Arc::new(
        worth_query_execution::facade::integration::WorthQueryGraphProviderAnchor::install::<ManagedGraph, _>(
            CheckpointContinuityProvider,
        ),
    );
        let provider_support = provider_anchor.resource_support().clone();
        let graph = worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::managed_run::tests::workflow_provider_steps::installed_graph(
            &installer,
            "checkpoint-continuity-graph",
            provider_anchor,
        );
        let runtime =
            worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::managed_run::tests::workflow_provider_steps::installed_runtime(installer, "checkpoint continuity");
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
            worth_query_testkit_execution_primary_graph_application_attempt::domain_computation::artifact_owner::authority_affinity_tests::installed_artifact_contract_for_managed_run(
            );
        let operation = workflow_authority_with_stage_graph_and_output_artifact(
            &runtime,
            &resources,
            "producer",
            &graph,
            WorthQueryOperationGraphAccess::Observe,
            output,
        );
        let running = worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::managed_run::tests::workflow_provider_steps::admitted_workflow(
            &runtime,
            &operation,
            resources,
            resource_request,
        );
        let installed_running_phase = execution.phase_for_workflow_for_test(&running);

        let production = running
            .artifacts()
            .production_authority("producer")
            .expect("producer artifact authority should resolve")
            .expect("producer output contract should be installed");
        let disposed = Arc::new(AtomicUsize::new(0));
        let admission = worth_query_execution::facade::domain_computation::WorthQueryArtifactProductionAuthority::admit(
            &production,
            WorthQueryArtifactProductionEvidence::new("preexisting", "preexisting-dependency"),
        );
        let preexisting =
            worth_query_execution::facade::domain_computation::WorthQueryArtifactProductionAuthority::register_exact(
                &production,
                admission,
                PreexistingArtifactResource(Arc::clone(&disposed)),
            )
            .expect("preexisting workflow artifact should register");

        let active = running
            .begin_stage_graph_execution(
                &installed_running_phase,
                "producer",
                &graph,
                WorthQueryManagedGraphCallRequest::new(
                    WorthQueryGraphProviderCallKind::Observe,
                    "checkpoint-continuity",
                ),
            )
            .expect("checkpoint provider should start");
        let active = match active.advance(&installed_running_phase) {
            WorthQueryWorkflowGraphStepOutcome::Continue(active) => active,
            _ => panic!("checkpoint-producing first step did not continue"),
        };
        let completion = match active.advance(&installed_running_phase) {
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

pub fn complete_observe_call(
    execution: &worth_query_execution::court_doors_for_test::domain_computation::primary_graph::request_execution::advancement::test_host_policy::TestHostRequest<'_>,

    running: worth_query_execution::facade::domain_computation::WorthQueryRunningWorkflowRun,
    graph: &WorthQueryInstalledGraphParticipationAuthority,
    scope: &str,
) -> worth_query_execution::facade::domain_computation::WorthQueryCompletedWorkflowGraphExecution {
    let active = {
        let installed_fixture_phase = running.fixture_phase_for_test(execution);
        running.begin_stage_graph_execution(
            &installed_fixture_phase,
            "producer",
            graph,
            WorthQueryManagedGraphCallRequest::new(WorthQueryGraphProviderCallKind::Observe, scope),
        )
    }
    .expect("multi-call provider should start");
    {
        let fixture_phase_result = {
            let installed_fixture_phase = active.fixture_phase_for_test(execution);
            active.advance(&installed_fixture_phase)
        };
        match fixture_phase_result {
            WorthQueryWorkflowGraphStepOutcome::Completed(completion) => completion,
            _ => panic!("multi-call provider did not complete"),
        }
    }
}

pub fn step_failure(
    denial: worth_query_execution::facade::domain_computation::WorthQueryGraphProviderStepDenial,
) -> WorthQueryGraphProviderFailure {
    WorthQueryGraphProviderFailure::new(denial.detail())
}
#[path = "artifact_retention/mod.rs"]
pub mod artifact_retention;
