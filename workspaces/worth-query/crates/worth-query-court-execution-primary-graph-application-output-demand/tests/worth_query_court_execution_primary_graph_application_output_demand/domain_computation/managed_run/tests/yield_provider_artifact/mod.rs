use std::collections::BTreeMap;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;

use crate::domain_computation::managed_run::tests::*;

pub struct CheckpointArtifactProvider {
    pub disposed: Arc<AtomicUsize>,
}

pub struct CheckpointArtifactExecution {
    pub disposed: Arc<AtomicUsize>,
    pub artifact:
        Option<worth_query_execution::facade::domain_computation::WorthQueryMoveOnlyArtifactHandle>,
    pub retained: Option<WorthQueryGraphProviderRetainedMemory>,
}

impl WorthQueryGraphProviderExecution for CheckpointArtifactExecution {
    fn advance(
        &mut self,
        step: &mut WorthQueryGraphProviderStep,
    ) -> Result<WorthQueryGraphProviderStepDisposition, WorthQueryGraphProviderFailure> {
        step.perform_work_unit(|| Ok(()))?;
        self.retained = Some(step.retain_bytes(5).map_err(step_failure)?);
        self.artifact = Some(
            step.produce_artifact(
                WorthQueryArtifactProductionEvidence::new(
                    "checkpoint-artifact-provider",
                    "checkpoint-artifact-dependency",
                ),
                CheckpointArtifactResource(Arc::clone(&self.disposed)),
            )
            .map_err(step_failure)?,
        );
        step.record_checkpoint_available().map_err(step_failure)?;
        Ok(WorthQueryGraphProviderStepDisposition::continue_work())
    }

    fn suspend(
        &mut self,
    ) -> Result<
        Box<dyn worth_query_execution::facade::domain_computation::WorthQueryGraphProviderCheckpoint>,
        WorthQueryGraphProviderFailure,
    >{
        Ok(Box::new(CheckpointOwningArtifact {
            artifact: self
                .artifact
                .take()
                .expect("governed provider step produced its checkpoint artifact"),
            retained: self
                .retained
                .take()
                .expect("checkpoint carries governed provider memory"),
        }))
    }

    fn dispose(&mut self) -> Result<(), WorthQueryGraphProviderFailure> {
        Ok(())
    }
}

impl WorthQueryGraphParticipationProvider<ManagedGraph> for CheckpointArtifactProvider {
    type Execution = CheckpointArtifactExecution;

    fn execution_resource_support(
        &self,
    ) -> worth_query_admission::facade::resource_admission::WorthQueryExecutionResourceSupport {
        worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::provider_session::tests::execution_resource_support_with_yield(
            "checkpoint-artifact-provider",
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
            CheckpointArtifactExecution {
                disposed: Arc::clone(&self.disposed),
                artifact: None,
                retained: None,
            },
        )
    }
}

pub struct CheckpointOwningArtifact {
    pub artifact:
        worth_query_execution::facade::domain_computation::WorthQueryMoveOnlyArtifactHandle,
    pub retained: WorthQueryGraphProviderRetainedMemory,
}

impl worth_query_execution::facade::domain_computation::WorthQueryGraphProviderCheckpoint
    for CheckpointOwningArtifact
{
    fn retained_bytes(&self) -> u64 {
        let _ = &self.artifact;
        u64::try_from(self.retained.len()).unwrap()
    }

    fn restore(
        &self,
        _call: &WorthQueryGraphProviderCall,
        _memory: &mut WorthQueryGraphProviderRestoreMemory,
    ) -> Result<
        WorthQueryCooperativeGraphProviderExecution<Box<dyn WorthQueryGraphProviderExecution>>,
        WorthQueryGraphProviderFailure,
    > {
        Err(WorthQueryGraphProviderFailure::new(
            "Phase 6.3 checkpoint-artifact proof must not restore",
        ))
    }
}

pub struct CheckpointArtifactResource(Arc<AtomicUsize>);

impl WorthQueryArtifactProviderResource for CheckpointArtifactResource {
    const PROVIDER_FAMILY: &'static str = "WORTH.tests.affinity.provider";

    fn canonical_semantic_projection(&self) -> Vec<u8> {
        b"checkpoint-owned-artifact".to_vec()
    }

    fn retained_bytes(&self) -> usize {
        32
    }

    fn dispose(&mut self) {
        self.0.fetch_add(1, Ordering::AcqRel);
    }
}

#[test]
fn yielded_cleanup_releases_artifacts_owned_by_the_provider_checkpoint() {
    worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::request_execution::advancement::with_test_advancement(|active_phase| {
        let bootstrap = active_phase.bootstrap_for_test();
        let resource_request = bootstrap.execution_request();

        let phase = &active_phase;
        let execution = phase;

        let installer = WorthQueryExecutionRuntimeInstaller::new();
        let disposed = Arc::new(AtomicUsize::new(0));
        let provider_anchor = Arc::new(
        worth_query_execution::facade::integration::WorthQueryGraphProviderAnchor::install::<ManagedGraph, _>(
            CheckpointArtifactProvider {
                disposed: Arc::clone(&disposed),
            },
        ),
    );
        let provider_support = provider_anchor.resource_support().clone();
        let graph = worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::managed_run::tests::workflow_provider_steps::installed_graph(
            &installer,
            "checkpoint-artifact-graph",
            provider_anchor,
        );
        let runtime =
            worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::managed_run::tests::workflow_provider_steps::installed_runtime(installer, "checkpoint artifact");
        let operation_resources = worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::provider_session::tests::admitted_yield_plan(
            "checkpoint-artifact",
            8,
        );
        let stage_resources = admitted_plan_with_graph_support(
            "checkpoint-artifact:producer",
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
        let active = {
            let installed_fixture_phase = running.fixture_phase_for_test(execution);
            running.begin_stage_graph_execution(
                &installed_fixture_phase,
                "producer",
                &graph,
                WorthQueryManagedGraphCallRequest::new(
                    WorthQueryGraphProviderCallKind::Observe,
                    "checkpoint-artifact-yield",
                ),
            )
        }
        .unwrap();
        let paused = {
            let fixture_phase_result = {
                let installed_fixture_phase = active.fixture_phase_for_test(execution);
                active.advance(&installed_fixture_phase)
            };
            match fixture_phase_result {
                WorthQueryWorkflowGraphStepOutcome::Continue(paused) => paused,
                _ => panic!("checkpoint-artifact provider did not pause"),
            }
        };
        let yielded = match paused.yield_run() {
            worth_query_execution::facade::domain_computation::WorthQueryWorkflowYieldOutcome::Yielded(yielded) => yielded,
            _ => panic!("checkpoint-artifact workflow did not yield"),
        };
        assert_eq!(
            yielded
                .inspection()
                .provider_work()
                .produced_artifact_count(),
            1
        );
        assert_eq!(
            yielded
                .inspection()
                .provider_work()
                .retained_artifact_count(),
            1
        );
        assert_eq!(yielded.inspection().provider_work().retained_bytes(), 37);
        assert_eq!(
            yielded
                .inspection()
                .artifact_evidence()
                .retained_artifact_count(),
            1
        );
        assert_eq!(
            yielded.inspection().provider_session_identity(),
            yielded
                .inspection()
                .provider_work()
                .provider_session_identity()
        );
        worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::managed_run::tests::cost_bound::assert_exact_admission_work(yielded.inspection().run_counters());
        assert_eq!(disposed.load(Ordering::Acquire), 0);

        let cleanup = match yielded.cleanup() {
            worth_query_execution::facade::domain_computation::WorthQueryWorkflowYieldCleanupOutcome::Complete(cleanup) => {
                cleanup
            }
            worth_query_execution::facade::domain_computation::WorthQueryWorkflowYieldCleanupOutcome::Pending(_) => {
                panic!("checkpoint-owned artifact formed a yielded cleanup ownership cycle")
            }
            worth_query_execution::facade::domain_computation::WorthQueryWorkflowYieldCleanupOutcome::RecoveryRequired(
                _,
            ) => {
                panic!("checkpoint-owned artifact release unexpectedly required recovery")
            }
        };
        assert_eq!(disposed.load(Ordering::Acquire), 1);
        let inspection = cleanup.inspection();
        assert_eq!(inspection.artifact_evidence().disposed_artifact_count(), 1);
        assert_eq!(inspection.provider_work().produced_artifact_count(), 1);
        assert_eq!(inspection.provider_work().retained_artifact_count(), 1);
        worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::managed_run::tests::cost_bound::assert_exact_admission_work(inspection.run_counters());
        assert!(inspection.resources_released());
        assert_eq!(inspection.released_reservation_count(), 3);
    });
}

pub fn step_failure(
    denial: worth_query_execution::facade::domain_computation::WorthQueryGraphProviderStepDenial,
) -> WorthQueryGraphProviderFailure {
    WorthQueryGraphProviderFailure::new(denial.detail())
}
