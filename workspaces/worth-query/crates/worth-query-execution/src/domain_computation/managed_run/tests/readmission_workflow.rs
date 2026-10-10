mod yielded_run;
use std::collections::BTreeMap;
use std::sync::Arc;
use yielded_run::yielded_workflow_fixture;

use super::yield_fixture::YieldProvider;
use super::*;

mod readmission_artifact;
pub(super) use readmission_artifact::ReadmissionArtifact;

pub(in crate::domain_computation::managed_run) fn yielded_workflow(
    execution: &crate::domain_computation::primary_graph::TestHostRequest<'_>,

    provider: YieldProvider,
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) -> (
    crate::domain_computation::WorthQueryYieldedWorkflowRun,
    RuntimeBridge,
    WorthQueryExecutionRuntime,
    Arc<crate::domain_computation::WorthQueryArtifactProductionAuthority>,
) {
    yielded_workflow_for_stage(execution, provider, "producer", resource_request)
}

pub(super) fn yielded_workflow_with_retained_artifact(
    execution: &crate::domain_computation::primary_graph::TestHostRequest<'_>,

    provider: YieldProvider,
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) -> (
    crate::domain_computation::WorthQueryYieldedWorkflowRun,
    RuntimeBridge,
    WorthQueryExecutionRuntime,
    Arc<crate::domain_computation::WorthQueryArtifactProductionAuthority>,
    crate::domain_computation::WorthQueryMoveOnlyArtifactHandle,
) {
    let (yielded, bridge, runtime, producer, artifact) =
        yielded_workflow_fixture(execution, provider, "producer", true, resource_request);
    (
        yielded,
        bridge,
        runtime,
        producer,
        artifact.expect("retained workflow artifact fixture must register"),
    )
}

pub(super) fn yielded_workflow_for_stage(
    execution: &crate::domain_computation::primary_graph::TestHostRequest<'_>,

    provider: YieldProvider,
    stage_identity: &str,
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) -> (
    crate::domain_computation::WorthQueryYieldedWorkflowRun,
    RuntimeBridge,
    WorthQueryExecutionRuntime,
    Arc<crate::domain_computation::WorthQueryArtifactProductionAuthority>,
) {
    let (yielded, bridge, runtime, producer, artifact) =
        yielded_workflow_fixture(execution, provider, stage_identity, false, resource_request);
    debug_assert!(artifact.is_none());
    (yielded, bridge, runtime, producer)
}

#[test]
fn workflow_readmission_rolls_generation_and_preserves_occurrence_state() {
    crate::domain_computation::primary_graph::with_test_advancement(|active_phase| {
        let bootstrap = active_phase.bootstrap_for_test();
        let resource_request = bootstrap.execution_request();

        let phase = &active_phase;
        let execution = phase;
        let active_request = execution;

        let (yielded, bridge, runtime, old_producer) =
            yielded_workflow(execution, YieldProvider::installed(7), resource_request);
        let logical = yielded.inspection().logical_run_identity().to_owned();
        let old_managed_attempt = yielded.inspection().yielded_attempt_identity().to_owned();
        let old_resource_attempt = yielded.inspection().yielded_attempt_identity().to_owned();
        let old_provider_session = yielded.inspection().provider_session_identity().to_owned();
        let old_artifacts = yielded.inspection().artifact_evidence();
        let old_provider_work = yielded.inspection().provider_work().clone();
        let reservations = yielded.inspection().retained_capacity_reservation_count();
        let readmitted = {
            let fixture_phase_result = {
                let installed_fixture_phase = yielded.fixture_phase(active_request);
                yielded.readmit_same_runtime(&installed_fixture_phase, &runtime, &bridge)
            };
            match fixture_phase_result {
                crate::domain_computation::WorthQueryWorkflowReadmissionOutcome::Readmitted(
                    readmitted,
                ) => readmitted,
                _ => panic!("same-runtime workflow readmission should succeed"),
            }
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

        let completion = {
            let fixture_phase_result = {
                let installed_fixture_phase = active.fixture_phase(execution);
                active.advance(&installed_fixture_phase)
            };
            match fixture_phase_result {
                WorthQueryWorkflowGraphStepOutcome::Completed(completion) => completion,
                _ => panic!("restored workflow provider did not complete"),
            }
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
        let bootstrap = active_phase.bootstrap_for_test();
        let resource_request = bootstrap.execution_request();

        let phase = &active_phase;
        let execution = phase;
        let active_request = execution;

        let (yielded, bridge, runtime, _producer) = yielded_workflow(
            execution,
            YieldProvider::checkpoint_restore_failure(7),
            resource_request,
        );
        let checkpoint = yielded.inspection().checkpoint().identity().to_owned();
        let generation = yielded
            .inspection()
            .artifact_evidence()
            .production_generation();
        let denied = {
            let fixture_phase_result = {
                let installed_fixture_phase = yielded.fixture_phase(active_request);
                yielded.readmit_same_runtime(&installed_fixture_phase, &runtime, &bridge)
            };
            match fixture_phase_result {
                crate::domain_computation::WorthQueryWorkflowReadmissionOutcome::Denied(denied) => {
                    denied
                }
                _ => panic!("ordinary workflow restore failure should deny"),
            }
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
        let bootstrap = active_phase.bootstrap_for_test();
        let resource_request = bootstrap.execution_request();

        let phase = &active_phase;
        let execution = phase;
        let active_request = execution;

        let (yielded, bridge, runtime, _producer) = yielded_workflow(
            execution,
            YieldProvider::checkpoint_restore_panic(7),
            resource_request,
        );
        let checkpoint = yielded.inspection().checkpoint().identity().to_owned();
        let generation = yielded
            .inspection()
            .artifact_evidence()
            .production_generation();
        let recovery = {
            let fixture_phase_result = {
                let installed_fixture_phase = yielded.fixture_phase(active_request);
                yielded.readmit_same_runtime(&installed_fixture_phase, &runtime, &bridge)
            };
            match fixture_phase_result {
            crate::domain_computation::WorthQueryWorkflowReadmissionOutcome::RecoveryRequired(
                recovery,
            ) => recovery,
            _ => panic!("workflow provider restore panic should require recovery"),
        }
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
