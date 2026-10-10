use super::*;

#[test]
fn composed_workflow_run_mints_artifact_authority_and_cleans_every_owner() {
    let host_request = crate::bridge_host_request::serial_request();
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = query_runtime();
    let operation_resources = admitted_plan("managed-workflow", 8);
    let stage_resources = admitted_plan("managed-workflow:stage", 4);
    let resources = WorthQueryAdmittedWorkflowResourcePlan::assemble(
        operation_resources,
        BTreeMap::from([("stage".to_owned(), stage_resources)]),
    );
    let operation = workflow_authority(&runtime, &resources);
    let attempt = runtime
        .start_workflow_resource_attempt(&operation, resources)
        .expect("workflow authority should reserve its exact resource plan");
    let lower = causal_fixture::managed_admission_context();

    let running = runtime
        .managed_run_admission(&lower.bridge, &lower.relational)
        .admit_workflow(&operation, attempt, lower.read_request(), resource_request)
        .expect("managed admission should compose workflow lower authorities")
        .start()
        .expect("workflow start should mint one artifact authority");
    assert!(!running.artifacts().run_identity().is_empty());
    let terminal = running
        .completed()
        .expect("workflow without provider work should complete");
    let terminal_identity = terminal.identity().to_owned();
    let logical_run_identity = terminal.logical_run_identity().to_owned();
    let provider_session_identity = terminal
        .provider_work()
        .provider_session_identity()
        .to_owned();
    super::cost_bound::assert_exact_admission_work(terminal.counters());
    let cleanup = complete_cleanup(terminal.cleanup());
    let inspection = cleanup.inspection();
    super::cost_bound::assert_exact_admission_work(inspection.counters());
    assert_eq!(inspection.run_identity(), terminal_identity);
    assert_eq!(inspection.logical_run_identity(), logical_run_identity);
    assert_eq!(
        inspection.provider_session_identity(),
        provider_session_identity
    );
    assert_eq!(
        inspection.disposition(),
        WorthQueryManagedRunCleanupDisposition::CleanupComplete
    );
    assert!(inspection.resources_released());
    assert_eq!(inspection.released_reservation_count(), 2);
    assert_artifact_evidence(inspection.artifact_evidence(), (0, 0, 0, 0));
}

#[test]
fn workflow_cleanup_pending_retains_run_until_live_artifact_owner_closes() {
    let host_request = crate::bridge_host_request::serial_request();
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = query_runtime();
    let operation_resources = admitted_plan("artifact-workflow", 8);
    let stage_resources = admitted_plan("artifact-workflow:producer", 4);
    let resources = WorthQueryAdmittedWorkflowResourcePlan::assemble(
        operation_resources,
        BTreeMap::from([("producer".to_owned(), stage_resources)]),
    );
    let output =
        crate::domain_computation::artifact_owner::installed_artifact_contract_for_managed_run();
    let operation =
        workflow_authority_with_output_artifact(&runtime, &resources, "producer", output);
    let attempt = runtime
        .start_workflow_resource_attempt(&operation, resources)
        .expect("artifact workflow should reserve");
    let lower = causal_fixture::managed_admission_context();
    let running = runtime
        .managed_run_admission(&lower.bridge, &lower.relational)
        .admit_workflow(&operation, attempt, lower.read_request(), resource_request)
        .expect("artifact workflow should admit")
        .start()
        .expect("artifact workflow should start");
    let production = running
        .artifacts()
        .production_authority("producer")
        .expect("stage should validate")
        .expect("stage should install output production authority");
    let admission =
        crate::domain_computation::artifact_owner::WorthQueryArtifactProductionAuthority::admit(
            &production,
            WorthQueryArtifactProductionEvidence::new("managed-provenance", "managed-dependency"),
        );
    let disposed = Arc::new(AtomicUsize::new(0));
    let handle =
        crate::domain_computation::artifact_owner::WorthQueryArtifactProductionAuthority::register_exact(
            &production,
            admission,
            PendingArtifactResource(Arc::clone(&disposed)),
        )
        .expect("exact production authority should register one artifact owner");
    let borrowed = handle
        .borrow("managed cleanup pending probe")
        .expect("installed contract should admit a shared read borrow");
    let terminal = running
        .completed()
        .expect("artifact production without uncertain provider calls may complete");
    let terminal_identity = terminal.identity().to_owned();
    let provider_session_identity = terminal
        .provider_work()
        .provider_session_identity()
        .to_owned();
    assert_artifact_evidence(terminal.artifact_evidence(), (1, 1, 0, 64));
    assert_eq!(terminal.provider_work().produced_artifact_count(), 0);
    let rejected_disposed = Arc::new(AtomicUsize::new(0));
    let rejected_admission =
        crate::domain_computation::artifact_owner::WorthQueryArtifactProductionAuthority::admit(
            &production,
            WorthQueryArtifactProductionEvidence::new(
                "post-terminal-provenance",
                "post-terminal-dependency",
            ),
        );
    let denial =
        match crate::domain_computation::artifact_owner::WorthQueryArtifactProductionAuthority::register_exact(
            &production,
            rejected_admission,
            PendingArtifactResource(Arc::clone(&rejected_disposed)),
        ) {
            Ok(_) => panic!("terminal workflow retained live artifact production authority"),
            Err(denial) => denial,
        };
    assert_eq!(
        denial.kind(),
        crate::domain_computation::WorthQueryArtifactDenialKind::ProductionClosed
    );
    let rejected_release = match denial.rejected_resource_release() {
        Some(crate::domain_computation::artifact_owner::WorthQueryArtifactProviderReleasePosture::Complete(
            evidence,
        )) => evidence,
        posture => panic!("rejected resource reported {posture:?}"),
    };
    assert_eq!(
        rejected_release.disposal(),
        crate::domain_computation::artifact_owner::WorthQueryArtifactProviderDisposalDisposition::Completed
    );
    assert_eq!(
        rejected_release.destructor(),
        crate::domain_computation::artifact_owner::WorthQueryArtifactProviderDestructorDisposition::Completed
    );
    assert_eq!(rejected_disposed.load(Ordering::Acquire), 1);

    let pending = match terminal.cleanup() {
        WorthQueryWorkflowRunCleanupOutcome::Pending(pending) => pending,
        WorthQueryWorkflowRunCleanupOutcome::Complete(_) => {
            panic!("live artifact borrow allowed CleanupComplete")
        }
        WorthQueryWorkflowRunCleanupOutcome::RecoveryRequired(failure) => {
            panic!("artifact close unexpectedly failed: {failure:?}")
        }
    };
    assert_eq!(pending.pending_artifact_owner_count(), 1);
    assert_artifact_evidence(pending.artifact_evidence(), (1, 1, 0, 64));
    assert_eq!(disposed.load(Ordering::Acquire), 0);

    drop(borrowed);
    assert_eq!(disposed.load(Ordering::Acquire), 1);
    let released_snapshot = handle.owner_snapshot();
    assert!(matches!(
        released_snapshot.provider_release(),
        crate::domain_computation::artifact_owner::WorthQueryArtifactProviderReleasePosture::Complete(_)
    ));
    assert_eq!(released_snapshot.counters().provider_disposals, 1);
    assert_eq!(released_snapshot.counters().provider_destructor_attempts, 1);
    assert_eq!(released_snapshot.counters().provider_release_failures, 0);
    drop(handle);
    let cleanup = complete_cleanup(pending.retry());
    let inspection = cleanup.inspection();
    assert_eq!(inspection.run_identity(), terminal_identity);
    assert_eq!(
        inspection.provider_session_identity(),
        provider_session_identity
    );
    assert_eq!(
        inspection.disposition(),
        WorthQueryManagedRunCleanupDisposition::CleanupComplete
    );
    assert!(inspection.resources_released());
    assert_artifact_evidence(inspection.artifact_evidence(), (1, 0, 1, 0));
    assert_eq!(
        inspection
            .artifact_evidence()
            .provider_release_complete_count(),
        1
    );
}

#[test]
fn workflow_cleanup_thread_failure_returns_the_same_terminal_for_retry() {
    let host_request = crate::bridge_host_request::serial_request();
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = query_runtime();
    let operation_resources = admitted_plan("workflow-cleanup-retry", 8);
    let stage_resources = admitted_plan("workflow-cleanup-retry:stage", 4);
    let resources = WorthQueryAdmittedWorkflowResourcePlan::assemble(
        operation_resources,
        BTreeMap::from([("stage".to_owned(), stage_resources)]),
    );
    let operation = workflow_authority(&runtime, &resources);
    let attempt = runtime
        .start_workflow_resource_attempt(&operation, resources)
        .expect("workflow resources should reserve");
    let lower = causal_fixture::managed_admission_context();
    let terminal = runtime
        .managed_run_admission(&lower.bridge, &lower.relational)
        .admit_workflow(&operation, attempt, lower.read_request(), resource_request)
        .expect("workflow run should admit")
        .start()
        .expect("workflow run should start")
        .completed()
        .expect("empty workflow should complete");
    let terminal_identity = terminal.identity().to_owned();
    let provider_session_identity = terminal
        .provider_work()
        .provider_session_identity()
        .to_owned();

    let failure = std::thread::spawn(move || match terminal.cleanup() {
        WorthQueryWorkflowRunCleanupOutcome::RecoveryRequired(failure) => failure,
        WorthQueryWorkflowRunCleanupOutcome::Complete(_) => {
            panic!("foreign thread terminalized workflow Signal")
        }
        WorthQueryWorkflowRunCleanupOutcome::Pending(_) => {
            panic!("workflow without artifacts reported pending cleanup")
        }
    })
    .join()
    .expect("cleanup probe should return its recovery authority");

    assert_eq!(
        failure.failure_kind(),
        WorthQueryManagedRunCleanupFailureKind::BridgeFinalization(
            BridgeExecutionBasisFinalizationFailureKind::SignalRuntimeThreadAffinityViolation
        )
    );
    assert_artifact_evidence(failure.artifact_evidence(), (0, 0, 0, 0));
    let cleanup = complete_cleanup(failure.retry());
    let inspection = cleanup.inspection();
    assert_eq!(inspection.run_identity(), terminal_identity);
    assert_eq!(
        inspection.provider_session_identity(),
        provider_session_identity
    );
    assert_eq!(
        inspection.disposition(),
        WorthQueryManagedRunCleanupDisposition::CleanupComplete
    );
    assert!(inspection.resources_released());
    assert_eq!(inspection.released_reservation_count(), 2);
    assert_artifact_evidence(inspection.artifact_evidence(), (0, 0, 0, 0));
}

#[test]
fn rejected_workflow_admission_returns_its_reserved_attempt() {
    let host_request = crate::bridge_host_request::serial_request();
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let owner_runtime = query_runtime();
    let foreign_runtime = query_runtime();
    let operation_resources = admitted_plan("rejected-workflow", 8);
    let stage_resources = admitted_plan("rejected-workflow:stage", 4);
    let resources = WorthQueryAdmittedWorkflowResourcePlan::assemble(
        operation_resources,
        BTreeMap::from([("stage".to_owned(), stage_resources)]),
    );
    let operation = workflow_authority(&owner_runtime, &resources);
    let attempt = owner_runtime
        .start_workflow_resource_attempt(&operation, resources)
        .expect("workflow resources should reserve");
    let lower = causal_fixture::managed_admission_context();

    let failure = match foreign_runtime
        .managed_run_admission(&lower.bridge, &lower.relational)
        .admit_workflow(&operation, attempt, lower.read_request(), resource_request)
    {
        Ok(_) => panic!("foreign Query runtime admitted workflow authority"),
        Err(failure) => failure,
    };
    assert_eq!(
        failure.kind(),
        crate::domain_computation::WorthQueryManagedWorkflowRunAdmissionFailureKind::QueryAuthority
    );
    let released = failure.into_resource_attempt().release();
    assert_eq!(
        released.capacity().scope(),
        WorthQueryExecutionCapacityReservationScope::Workflow
    );
    assert_eq!(released.capacity().released_reservation_count(), 2);
}

fn complete_cleanup(
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

fn assert_artifact_evidence(
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

struct PendingArtifactResource(Arc<AtomicUsize>);

impl WorthQueryArtifactProviderResource for PendingArtifactResource {
    const PROVIDER_FAMILY: &'static str = "WORTH.tests.affinity.provider";

    fn canonical_semantic_projection(&self) -> Vec<u8> {
        b"managed-artifact".to_vec()
    }

    fn retained_bytes(&self) -> usize {
        64
    }

    fn dispose(&mut self) {
        self.0.fetch_add(1, Ordering::AcqRel);
    }
}
