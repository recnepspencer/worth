use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::managed_run::tests::yield_fixture::YieldProvider;
use crate::domain_computation::managed_run::tests::*;

#[test]
fn failed_workflow_yield_reports_pending_artifacts_and_preserves_retry_authority() {
    worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::request_execution::advancement::with_test_advancement(|active_phase| {
        let bootstrap = active_phase.bootstrap_for_test();
        let resource_request = bootstrap.execution_request();

        let phase = &active_phase;
        let execution = phase;

        let disposals = Arc::new(AtomicUsize::new(0));
        let world = terminalized_recovery_world(
            "workflow-yield-recovery-artifact-graph",
            RecoveryArtifactResource(Arc::clone(&disposals)),
            resource_request,
        );
        let TerminalizedRecoveryWorld {
            running,
            graph,
            handle,
        } = world;
        let borrowed = handle
            .borrow("failed-yield cleanup ownership probe")
            .expect("installed artifact contract should admit a shared borrow");
        let recovery = terminalize_recovery_world(execution, running, &graph);
        let initial_artifacts = recovery
            .artifact_evidence()
            .expect("terminalized workflow recovery reports artifact ownership");
        assert_eq!(initial_artifacts.produced_artifact_count(), 1);
        assert_eq!(initial_artifacts.retained_artifact_count(), 1);
        assert_eq!(initial_artifacts.retained_bytes(), 64);
        assert_eq!(
        recovery.kind(),
        worth_query_execution::facade::domain_computation::WorthQueryYieldRecoveryKind::ProviderCheckpointSuspension(
            worth_query_execution::facade::domain_computation::WorthQueryProviderCheckpointSuspensionFailureKind::ProviderRejected,
        )
    );

        let pending = match recovery.release_terminalized() {
        Ok(worth_query_execution::facade::domain_computation::WorthQueryWorkflowYieldRecoveryReleaseOutcome::Pending(
            pending,
        )) => pending,
        Ok(worth_query_execution::facade::domain_computation::WorthQueryWorkflowYieldRecoveryReleaseOutcome::Complete(
            _,
        )) => {
            panic!("outstanding artifact borrow allowed complete failed-yield cleanup")
        }
        Ok(
            worth_query_execution::facade::domain_computation::WorthQueryWorkflowYieldRecoveryReleaseOutcome::RecoveryRequired(_),
        ) => panic!("outstanding artifact borrow skipped the pending cleanup phase"),
        Err(_) => panic!("terminalized failed yield was misclassified as running"),
    };
        assert_eq!(disposals.load(Ordering::Acquire), 0);

        drop(borrowed);
        assert_eq!(disposals.load(Ordering::Acquire), 1);
        drop(handle);
        let release = match pending.retry() {
        Ok(worth_query_execution::facade::domain_computation::WorthQueryWorkflowYieldRecoveryReleaseOutcome::Complete(
            release,
        )) => release,
        Ok(worth_query_execution::facade::domain_computation::WorthQueryWorkflowYieldRecoveryReleaseOutcome::Pending(
            _,
        )) => {
            panic!("released artifact owner kept failed-yield cleanup pending")
        }
        Ok(
            worth_query_execution::facade::domain_computation::WorthQueryWorkflowYieldRecoveryReleaseOutcome::RecoveryRequired(_),
        ) => panic!("successful artifact release required recovery"),
        Err(_) => panic!("pending failed-yield cleanup lost terminalized retry authority"),
    };
        let inspection = release.inspection();
        assert_eq!(inspection.artifact_evidence().disposed_artifact_count(), 1);
        assert_eq!(
            inspection
                .artifact_evidence()
                .provider_release_complete_count(),
            1
        );
        assert!(inspection.resources_released());
        assert_eq!(inspection.released_reservation_count(), 3);
    });
}

#[test]
fn terminalized_workflow_yield_types_double_artifact_release_panic_as_recovery() {
    worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::request_execution::advancement::with_test_advancement(|active_phase| {
        let bootstrap = active_phase.bootstrap_for_test();
        let resource_request = bootstrap.execution_request();

        let phase = &active_phase;
        let execution = phase;

        let disposal_attempts = Arc::new(AtomicUsize::new(0));
        let destructor_attempts = Arc::new(AtomicUsize::new(0));
        let world = terminalized_recovery_world(
            "workflow-yield-recovery-double-panic",
            DoublePanickingRecoveryArtifactResource {
                disposal_attempts: Arc::clone(&disposal_attempts),
                destructor_attempts: Arc::clone(&destructor_attempts),
            },
            resource_request,
        );
        let TerminalizedRecoveryWorld {
            running,
            graph,
            handle,
        } = world;
        let recovery = terminalize_recovery_world(execution, running, &graph);
        assert_eq!(
        recovery.kind(),
        worth_query_execution::facade::domain_computation::WorthQueryYieldRecoveryKind::ProviderCheckpointSuspension(
            worth_query_execution::facade::domain_computation::WorthQueryProviderCheckpointSuspensionFailureKind::ProviderRejected,
        )
    );
        let release = match recovery.release_terminalized() {
        Ok(
            worth_query_execution::facade::domain_computation::WorthQueryWorkflowYieldRecoveryReleaseOutcome::RecoveryRequired(release),
        ) => release,
        Ok(worth_query_execution::facade::domain_computation::WorthQueryWorkflowYieldRecoveryReleaseOutcome::Complete(
            _,
        )) => panic!("double artifact release panic was reported as complete"),
        Ok(worth_query_execution::facade::domain_computation::WorthQueryWorkflowYieldRecoveryReleaseOutcome::Pending(
            _,
        )) => panic!("artifact without a surviving borrow remained pending"),
        Err(_) => panic!("terminalized failed yield lost release authority"),
    };
        let inspection = release.inspection();
        assert_eq!(
            inspection
                .artifact_evidence()
                .provider_release_recovery_required_count(),
            1
        );
        assert_eq!(disposal_attempts.load(Ordering::Acquire), 1);
        assert_eq!(destructor_attempts.load(Ordering::Acquire), 1);
        assert!(inspection.resources_released());
        assert_eq!(inspection.released_reservation_count(), 3);
        let artifact_release = match handle.owner_snapshot().provider_release() {
        worth_query_execution::facade::domain_computation::WorthQueryArtifactProviderReleasePosture::RecoveryRequired(
            evidence,
        ) => evidence,
        posture => panic!("terminalized artifact release reported {posture:?}"),
    };
        assert_eq!(
        artifact_release.disposal(),
        worth_query_execution::facade::domain_computation::WorthQueryArtifactProviderDisposalDisposition::Panicked
    );
        assert_eq!(
        artifact_release.destructor(),
        worth_query_execution::facade::domain_computation::WorthQueryArtifactProviderDestructorDisposition::Panicked
    );
    });
}

pub struct TerminalizedRecoveryWorld {
    pub running: worth_query_execution::facade::domain_computation::WorthQueryRunningWorkflowRun,
    pub graph: WorthQueryInstalledGraphParticipationAuthority,
    pub handle: worth_query_execution::facade::domain_computation::WorthQueryMoveOnlyArtifactHandle,
}

pub fn terminalized_recovery_world<R>(
    label: &str,
    resource: R,
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) -> TerminalizedRecoveryWorld
where
    R: WorthQueryArtifactProviderResource,
{
    let installer = WorthQueryExecutionRuntimeInstaller::new();
    let provider_anchor = Arc::new(
        worth_query_execution::facade::integration::WorthQueryGraphProviderAnchor::install::<
            ManagedGraph,
            _,
        >(YieldProvider::suspension_failure()),
    );
    let provider_support = provider_anchor.resource_support().clone();
    let graph = worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::managed_run::tests::workflow_provider_steps::installed_graph(
        &installer,
        &format!("{label}:graph"),
        provider_anchor,
    );
    let runtime =
        worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::managed_run::tests::workflow_provider_steps::installed_runtime(installer, &format!("{label}:runtime"));
    let operation_resources =
        worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::provider_session::tests::admitted_yield_plan(label, 8);
    let stage_resources = admitted_plan_with_graph_support(
        &format!("{label}:producer"),
        8,
        graph.role(),
        provider_support,
    );
    let resources = WorthQueryAdmittedWorkflowResourcePlan::assemble(
        operation_resources,
        BTreeMap::from([("producer".to_owned(), stage_resources)]),
    );
    let output =
        worth_query_testkit_execution_primary_graph_application_attempt::domain_computation::artifact_owner::authority_affinity_tests::installed_artifact_contract_for_managed_run();
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
    let production = running
        .artifacts()
        .production_authority("producer")
        .expect("producer role should validate")
        .expect("producer output contract should install");
    let admission =
        worth_query_execution::facade::domain_computation::WorthQueryArtifactProductionAuthority::admit(
            &production,
            WorthQueryArtifactProductionEvidence::new(
                format!("{label}:provenance"),
                format!("{label}:dependency"),
            ),
        );
    let handle =
        worth_query_execution::facade::domain_computation::WorthQueryArtifactProductionAuthority::register_exact(
            &production,
            admission,
            resource,
        )
        .expect("recovery artifact should register before yield begins");
    TerminalizedRecoveryWorld {
        running,
        graph,
        handle,
    }
}

pub fn terminalize_recovery_world(
    execution: &worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::request_execution::advancement::test_host_policy::TestHostRequest<'_>,

    running: worth_query_execution::facade::domain_computation::WorthQueryRunningWorkflowRun,
    graph: &WorthQueryInstalledGraphParticipationAuthority,
) -> worth_query_execution::facade::domain_computation::WorthQueryWorkflowYieldRecoveryRequired {
    let active = {
        let installed_fixture_phase = running.fixture_phase(execution);
        running.begin_stage_graph_execution(
            &installed_fixture_phase,
            "producer",
            graph,
            WorthQueryManagedGraphCallRequest::new(
                WorthQueryGraphProviderCallKind::Observe,
                "workflow-yield-recovery",
            ),
        )
    }
    .expect("recovery provider should begin");
    let paused = {
        let fixture_phase_result = {
            let installed_fixture_phase = active.fixture_phase(execution);
            active.advance(&installed_fixture_phase)
        };
        match fixture_phase_result {
            WorthQueryWorkflowGraphStepOutcome::Continue(paused) => paused,
            _ => panic!("recovery provider did not reach its safe point"),
        }
    };

    match paused.yield_run() {
        worth_query_execution::facade::domain_computation::WorthQueryWorkflowYieldOutcome::RecoveryRequired(recovery) => {
            recovery
        }
        _ => panic!("provider suspension failure did not return recovery authority"),
    }
}

pub struct RecoveryArtifactResource(Arc<AtomicUsize>);

impl WorthQueryArtifactProviderResource for RecoveryArtifactResource {
    const PROVIDER_FAMILY: &'static str = "WORTH.tests.affinity.provider";

    fn canonical_semantic_projection(&self) -> Vec<u8> {
        b"yield-recovery-artifact".to_vec()
    }

    fn retained_bytes(&self) -> usize {
        64
    }

    fn dispose(&mut self) {
        self.0.fetch_add(1, Ordering::AcqRel);
    }
}

pub struct DoublePanickingRecoveryArtifactResource {
    pub disposal_attempts: Arc<AtomicUsize>,
    pub destructor_attempts: Arc<AtomicUsize>,
}

impl WorthQueryArtifactProviderResource for DoublePanickingRecoveryArtifactResource {
    const PROVIDER_FAMILY: &'static str = "WORTH.tests.affinity.provider";

    fn canonical_semantic_projection(&self) -> Vec<u8> {
        b"terminalized-yield-double-panic".to_vec()
    }

    fn retained_bytes(&self) -> usize {
        64
    }

    fn dispose(&mut self) {
        self.disposal_attempts.fetch_add(1, Ordering::AcqRel);
        panic!("terminalized artifact provider disposal panicked")
    }
}

impl Drop for DoublePanickingRecoveryArtifactResource {
    fn drop(&mut self) {
        self.destructor_attempts.fetch_add(1, Ordering::AcqRel);
        panic!("terminalized artifact provider destructor panicked")
    }
}
