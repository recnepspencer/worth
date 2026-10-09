use std::collections::BTreeMap;
use std::sync::Arc;

use super::yield_fixture::YieldProvider;
use super::*;

pub(super) struct ReadmissionArtifact;

impl WorthQueryArtifactProviderResource for ReadmissionArtifact {
    const PROVIDER_FAMILY: &'static str = "WORTH.tests.affinity.provider";

    fn canonical_semantic_projection(&self) -> Vec<u8> {
        b"workflow-readmission-artifact".to_vec()
    }

    fn retained_bytes(&self) -> usize {
        9
    }

    fn dispose(&mut self) {}
}

pub(in crate::domain_computation::managed_run) fn yielded_workflow(
    execution: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,

    provider: YieldProvider,
) -> (
    crate::domain_computation::WorthQueryYieldedWorkflowRun,
    RuntimeBridge,
    WorthQueryExecutionRuntime,
    Arc<crate::domain_computation::WorthQueryArtifactProductionAuthority>,
) {
    yielded_workflow_for_stage(execution, provider, "producer")
}

pub(super) fn yielded_workflow_with_retained_artifact(
    execution: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,

    provider: YieldProvider,
) -> (
    crate::domain_computation::WorthQueryYieldedWorkflowRun,
    RuntimeBridge,
    WorthQueryExecutionRuntime,
    Arc<crate::domain_computation::WorthQueryArtifactProductionAuthority>,
    crate::domain_computation::WorthQueryMoveOnlyArtifactHandle,
) {
    let (yielded, bridge, runtime, producer, artifact) =
        yielded_workflow_fixture(execution, provider, "producer", true);
    (
        yielded,
        bridge,
        runtime,
        producer,
        artifact.expect("retained workflow artifact fixture must register"),
    )
}

pub(super) fn yielded_workflow_for_stage(
    execution: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,

    provider: YieldProvider,
    stage_identity: &str,
) -> (
    crate::domain_computation::WorthQueryYieldedWorkflowRun,
    RuntimeBridge,
    WorthQueryExecutionRuntime,
    Arc<crate::domain_computation::WorthQueryArtifactProductionAuthority>,
) {
    let (yielded, bridge, runtime, producer, artifact) =
        yielded_workflow_fixture(execution, provider, stage_identity, false);
    debug_assert!(artifact.is_none());
    (yielded, bridge, runtime, producer)
}

fn yielded_workflow_fixture(
    execution: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,

    provider: YieldProvider,
    stage_identity: &str,
    retain_artifact: bool,
) -> (
    crate::domain_computation::WorthQueryYieldedWorkflowRun,
    RuntimeBridge,
    WorthQueryExecutionRuntime,
    Arc<crate::domain_computation::WorthQueryArtifactProductionAuthority>,
    Option<crate::domain_computation::WorthQueryMoveOnlyArtifactHandle>,
) {
    let installer = WorthQueryExecutionRuntimeInstaller::new();
    let provider_anchor = Arc::new(
        crate::domain_computation::provider_session::graph_provider::bounded_step::provider_anchor::WorthQueryGraphProviderAnchor::install::<ManagedGraph, _>(
            provider,
        ),
    );
    let provider_support = provider_anchor.resource_support().clone();
    let graph = super::workflow_provider_steps::installed_graph(
        &installer,
        "workflow-readmission-graph",
        provider_anchor,
    );
    let runtime =
        super::workflow_provider_steps::installed_runtime(installer, "workflow readmission");
    let operation_resources =
        crate::domain_computation::provider_session::admitted_yield_plan("workflow-readmission", 8);
    let stage_resource_identity = format!("workflow-readmission:{stage_identity}");
    let stage_resources = admitted_plan_with_graph_support(
        &stage_resource_identity,
        8,
        graph.role(),
        provider_support,
    );
    let resources = WorthQueryAdmittedWorkflowResourcePlan::assemble(
        operation_resources,
        BTreeMap::from([(stage_identity.to_owned(), stage_resources)]),
    );
    let output =
        crate::domain_computation::artifact_owner::installed_artifact_contract_for_managed_run();
    let operation = workflow_authority_with_stage_graph_and_output_artifact(
        &runtime,
        &resources,
        stage_identity,
        &graph,
        WorthQueryOperationGraphAccess::Observe,
        output,
    );
    let attempt = runtime
        .start_workflow_resource_attempt(&operation, resources)
        .expect("workflow resources should reserve");
    let lower = causal_fixture::managed_admission_context();
    let running = runtime
        .managed_run_admission(&lower.bridge, &lower.relational)
        .admit_workflow(&operation, attempt, lower.read_request())
        .expect("workflow should admit")
        .start()
        .expect("workflow should start");
    let old_producer = running
        .artifacts()
        .production_authority(stage_identity)
        .expect("producer stage should validate")
        .expect("producer stage should own output authority");
    let retained_artifact = retain_artifact.then(|| {
        let admission = crate::domain_computation::WorthQueryArtifactProductionAuthority::admit(
            &old_producer,
            WorthQueryArtifactProductionEvidence::new("readmission-cleanup", "retained-artifact"),
        );
        crate::domain_computation::WorthQueryArtifactProductionAuthority::register_exact(
            &old_producer,
            admission,
            ReadmissionArtifact,
        )
        .expect("retained workflow artifact must register before production freezes")
    });
    let active = running
        .begin_stage_graph_execution(
            execution,
            stage_identity,
            &graph,
            WorthQueryManagedGraphCallRequest::new(
                WorthQueryGraphProviderCallKind::Observe,
                "workflow-readmission",
            ),
        )
        .expect("workflow provider should begin");
    let paused = match active.advance(execution) {
        WorthQueryWorkflowGraphStepOutcome::Continue(paused) => paused,
        _ => panic!("workflow provider did not pause"),
    };
    let yielded = match paused.yield_run() {
        crate::domain_computation::WorthQueryWorkflowYieldOutcome::Yielded(yielded) => yielded,
        _ => panic!("eligible workflow did not yield"),
    };
    (
        yielded,
        lower.bridge,
        runtime,
        old_producer,
        retained_artifact,
    )
}

#[test]
fn workflow_readmission_rolls_generation_and_preserves_occurrence_state() {
    crate::domain_computation::primary_graph::with_test_advancement(|active_phase| {
        let phase = &active_phase;
        let execution = phase;
        let active_request = execution;

        let (yielded, bridge, runtime, old_producer) =
            yielded_workflow(execution, YieldProvider::installed(7));
        let logical = yielded.inspection().logical_run_identity().to_owned();
        let old_managed_attempt = yielded.inspection().yielded_attempt_identity().to_owned();
        let old_resource_attempt = yielded.inspection().yielded_attempt_identity().to_owned();
        let old_provider_session = yielded.inspection().provider_session_identity().to_owned();
        let old_artifacts = yielded.inspection().artifact_evidence();
        let old_provider_work = yielded.inspection().provider_work().clone();
        let reservations = yielded.inspection().retained_capacity_reservation_count();
        let readmitted = match yielded.readmit_same_runtime(active_request, &runtime, &bridge) {
            crate::domain_computation::WorthQueryWorkflowReadmissionOutcome::Readmitted(
                readmitted,
            ) => readmitted,
            _ => panic!("same-runtime workflow readmission should succeed"),
        };
        let active = readmitted.into_active();
        assert_eq!(active.logical_run_identity(), logical);
        assert_ne!(active.run_identity(), old_managed_attempt);
        assert_ne!(active.resource_attempt_identity(), old_resource_attempt);
        assert_ne!(active.provider_session_identity(), old_provider_session);
        assert_eq!(active.retained_capacity_reservation_count(), reservations);
        assert_eq!(reservations, 3);
        let fresh_artifacts = active.artifact_evidence();
        assert_eq!(
            fresh_artifacts.production_generation(),
            old_artifacts.production_generation() + 1
        );
        assert_eq!(
            (
                fresh_artifacts.produced_artifact_count(),
                fresh_artifacts.retained_artifact_count(),
                fresh_artifacts.disposed_artifact_count(),
                fresh_artifacts.retained_bytes(),
            ),
            (
                old_artifacts.produced_artifact_count(),
                old_artifacts.retained_artifact_count(),
                old_artifacts.disposed_artifact_count(),
                old_artifacts.retained_bytes(),
            )
        );

        let old_admission = crate::domain_computation::WorthQueryArtifactProductionAuthority::admit(
            &old_producer,
            WorthQueryArtifactProductionEvidence::new("old-generation", "readmission"),
        );
        let old_denial =
            crate::domain_computation::WorthQueryArtifactProductionAuthority::register_exact(
                &old_producer,
                old_admission,
                ReadmissionArtifact,
            )
            .expect_err("pre-yield producer must remain stale");
        assert_eq!(
            old_denial.kind(),
            crate::domain_computation::WorthQueryArtifactDenialKind::StaleLifecycleGeneration
        );
        let fresh_producer = active
            .artifacts()
            .production_authority("producer")
            .expect("fresh producer stage should validate")
            .expect("fresh generation should mint production authority");
        let fresh_admission =
            crate::domain_computation::WorthQueryArtifactProductionAuthority::admit(
                &fresh_producer,
                WorthQueryArtifactProductionEvidence::new("fresh-generation", "readmission"),
            );
        let fresh_handle =
            crate::domain_computation::WorthQueryArtifactProductionAuthority::register_exact(
                &fresh_producer,
                fresh_admission,
                ReadmissionArtifact,
            )
            .expect("fresh producer should register");
        drop(fresh_handle);

        let completion = match active.advance(execution) {
            WorthQueryWorkflowGraphStepOutcome::Completed(completion) => completion,
            _ => panic!("restored workflow provider did not complete"),
        };
        let terminal = completion.into_running().completed().unwrap();
        assert_eq!(terminal.logical_run_identity(), logical);
        assert_eq!(terminal.provider_work().completed_work_units(), 4);
        assert_eq!(
            terminal.provider_work().produced_artifact_count(),
            old_provider_work.produced_artifact_count()
        );
        match terminal.cleanup() {
            WorthQueryWorkflowRunCleanupOutcome::Complete(_) => {}
            _ => panic!("readmitted workflow did not clean up"),
        }
    });
}

#[test]
fn workflow_provider_restore_denial_keeps_frozen_generation_retryable() {
    crate::domain_computation::primary_graph::with_test_advancement(|active_phase| {
        let phase = &active_phase;
        let execution = phase;
        let active_request = execution;

        let (yielded, bridge, runtime, _producer) =
            yielded_workflow(execution, YieldProvider::checkpoint_restore_failure(7));
        let checkpoint = yielded.inspection().checkpoint().identity().to_owned();
        let generation = yielded
            .inspection()
            .artifact_evidence()
            .production_generation();
        let denied = match yielded.readmit_same_runtime(active_request, &runtime, &bridge) {
            crate::domain_computation::WorthQueryWorkflowReadmissionOutcome::Denied(denied) => {
                denied
            }
            _ => panic!("ordinary workflow restore failure should deny"),
        };
        assert_eq!(
        denied.kind(),
        crate::domain_computation::WorthQueryWorkflowReadmissionDenialKind::ProviderRestoreDenied
    );
        let counters = denied.readmission_evidence().query_counters();
        assert_eq!(counters.artifact_generation_attempt_count(), 0);
        let yielded = denied.into_yielded();
        assert_eq!(yielded.inspection().checkpoint().identity(), checkpoint);
        assert_eq!(
            yielded
                .inspection()
                .artifact_evidence()
                .production_generation(),
            generation
        );
        match yielded.cleanup() {
            crate::domain_computation::WorthQueryWorkflowYieldCleanupOutcome::Complete(_) => {}
            _ => panic!("denied workflow should retain complete cleanup authority"),
        }
    });
}

#[test]
fn workflow_restore_panic_can_recover_only_through_terminal_cleanup() {
    crate::domain_computation::primary_graph::with_test_advancement(|active_phase| {
        let phase = &active_phase;
        let execution = phase;
        let active_request = execution;

        let (yielded, bridge, runtime, _producer) =
            yielded_workflow(execution, YieldProvider::checkpoint_restore_panic(7));
        let checkpoint = yielded.inspection().checkpoint().identity().to_owned();
        let generation = yielded
            .inspection()
            .artifact_evidence()
            .production_generation();
        let recovery = match yielded.readmit_same_runtime(active_request, &runtime, &bridge) {
            crate::domain_computation::WorthQueryWorkflowReadmissionOutcome::RecoveryRequired(
                recovery,
            ) => recovery,
            _ => panic!("workflow provider restore panic should require recovery"),
        };
        assert_eq!(
        recovery.posture(),
        crate::domain_computation::WorthQueryWorkflowReadmissionRecoveryPosture::
            TerminalCleanupRequired
    );
        let recovery = match recovery {
        crate::domain_computation::WorthQueryWorkflowReadmissionRecoveryRequired::TerminalCleanup(
            recovery,
        ) => recovery,
        _ => panic!("provider panic must not expose workflow yield-reassembly authority"),
    };
        let receipt = match recovery.into_cleanup().finish() {
            crate::domain_computation::WorthQueryWorkflowReadmissionCleanupOutcome::Complete(
                receipt,
            ) => receipt,
            _ => panic!("retained workflow authorities should complete terminal cleanup"),
        };
        let inspection = receipt.inspection();
        assert_eq!(inspection.checkpoint().identity(), checkpoint);
        assert_eq!(
            inspection.artifact_evidence().production_generation(),
            generation
        );
        assert!(inspection.resources_released());
    });
}

#[path = "readmission_workflow/workflow_restore_rejection_after_admission_is_terminal_even_after_clean_release.rs"]
mod workflow_restore_rejection_after_admission_is_terminal_even_after_clean_release;
