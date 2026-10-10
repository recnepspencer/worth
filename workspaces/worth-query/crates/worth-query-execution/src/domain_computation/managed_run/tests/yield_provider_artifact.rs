use std::collections::BTreeMap;
use std::sync::atomic::{AtomicUsize, Ordering};

use super::*;

struct CheckpointArtifactProvider {
    disposed: Arc<AtomicUsize>,
}

struct CheckpointArtifactExecution {
    disposed: Arc<AtomicUsize>,
    artifact: Option<crate::domain_computation::WorthQueryMoveOnlyArtifactHandle>,
    retained: Option<WorthQueryGraphProviderRetainedMemory>,
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
        Box<dyn crate::domain_computation::WorthQueryGraphProviderCheckpoint>,
        WorthQueryGraphProviderFailure,
    > {
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
        crate::domain_computation::provider_session::execution_resource_support_with_yield(
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

struct CheckpointOwningArtifact {
    artifact: crate::domain_computation::WorthQueryMoveOnlyArtifactHandle,
    retained: WorthQueryGraphProviderRetainedMemory,
}

impl crate::domain_computation::WorthQueryGraphProviderCheckpoint for CheckpointOwningArtifact {
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

struct CheckpointArtifactResource(Arc<AtomicUsize>);

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
    crate::domain_computation::primary_graph::with_test_advancement(|active_phase| {
        let bootstrap = active_phase.bootstrap_for_test();
        let resource_request = bootstrap.execution_request();

        let phase = &active_phase;
        let execution = phase;

        let installer = WorthQueryExecutionRuntimeInstaller::new();
        let disposed = Arc::new(AtomicUsize::new(0));
        let provider_anchor = Arc::new(
        crate::domain_computation::provider_session::graph_provider::bounded_step::provider_anchor::WorthQueryGraphProviderAnchor::install::<ManagedGraph, _>(
            CheckpointArtifactProvider {
                disposed: Arc::clone(&disposed),
            },
        ),
    );
        let provider_support = provider_anchor.resource_support().clone();
        let graph = super::workflow_provider_steps::installed_graph(
            &installer,
            "checkpoint-artifact-graph",
            provider_anchor,
        );
        let runtime =
            super::workflow_provider_steps::installed_runtime(installer, "checkpoint artifact");
        let operation_resources = crate::domain_computation::provider_session::admitted_yield_plan(
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
        let active = running
            .begin_stage_graph_execution(
                execution,
                "producer",
                &graph,
                WorthQueryManagedGraphCallRequest::new(
                    WorthQueryGraphProviderCallKind::Observe,
                    "checkpoint-artifact-yield",
                ),
            )
            .unwrap();
        let paused = match active.advance(execution) {
            WorthQueryWorkflowGraphStepOutcome::Continue(paused) => paused,
            _ => panic!("checkpoint-artifact provider did not pause"),
        };
        let yielded = match paused.yield_run() {
            crate::domain_computation::WorthQueryWorkflowYieldOutcome::Yielded(yielded) => yielded,
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
        super::cost_bound::assert_exact_admission_work(yielded.inspection().run_counters());
        assert_eq!(disposed.load(Ordering::Acquire), 0);

        let cleanup = match yielded.cleanup() {
            crate::domain_computation::WorthQueryWorkflowYieldCleanupOutcome::Complete(cleanup) => {
                cleanup
            }
            crate::domain_computation::WorthQueryWorkflowYieldCleanupOutcome::Pending(_) => {
                panic!("checkpoint-owned artifact formed a yielded cleanup ownership cycle")
            }
            crate::domain_computation::WorthQueryWorkflowYieldCleanupOutcome::RecoveryRequired(
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
        super::cost_bound::assert_exact_admission_work(inspection.run_counters());
        assert!(inspection.resources_released());
        assert_eq!(inspection.released_reservation_count(), 3);
    });
}

fn step_failure(
    denial: crate::domain_computation::WorthQueryGraphProviderStepDenial,
) -> WorthQueryGraphProviderFailure {
    WorthQueryGraphProviderFailure::new(denial.detail())
}
