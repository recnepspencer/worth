use super::*;

#[test]
fn workflow_cleanup_contains_artifact_disposal_and_destructor_panics() {
    let world = double_panicking_artifact_world("artifact-double-panic");

    let terminal = world
        .running
        .completed()
        .expect("owned artifact does not prevent semantic completion");
    let cleanup = match terminal.cleanup() {
        WorthQueryWorkflowRunCleanupOutcome::Complete(receipt) => receipt,
        WorthQueryWorkflowRunCleanupOutcome::Pending(_) => {
            panic!("owner without borrows remained pending after registry close")
        }
        WorthQueryWorkflowRunCleanupOutcome::RecoveryRequired(failure) => {
            panic!("provider panic escaped into lower-layer cleanup failure: {failure:?}")
        }
    };
    assert_eq!(
        cleanup.inspection().disposition(),
        WorthQueryManagedRunCleanupDisposition::RecoveryRequired
    );
    assert!(cleanup.inspection().resources_released());
    assert_eq!(cleanup.inspection().released_reservation_count(), 2);
    let evidence = cleanup.inspection().artifact_evidence();
    assert_eq!(evidence.produced_artifact_count(), 1);
    assert_eq!(evidence.retained_artifact_count(), 0);
    assert_eq!(evidence.disposed_artifact_count(), 1);
    assert_eq!(evidence.retained_bytes(), 0);
    assert_eq!(evidence.provider_release_complete_count(), 0);
    assert_eq!(evidence.provider_release_pending_count(), 0);
    assert_eq!(evidence.provider_release_recovery_required_count(), 1);
    assert_eq!(world.disposal_attempts.load(Ordering::Acquire), 1);
    assert_eq!(world.destructor_attempts.load(Ordering::Acquire), 1);
    let release = match world.handle.owner_snapshot().provider_release() {
        crate::domain_computation::artifact_owner::WorthQueryArtifactProviderReleasePosture::RecoveryRequired(
            evidence,
        ) => evidence,
        posture => panic!("double-panic artifact reported {posture:?}"),
    };
    assert_eq!(
        release.disposal(),
        crate::domain_computation::artifact_owner::WorthQueryArtifactProviderDisposalDisposition::Panicked
    );
    assert_eq!(
        release.destructor(),
        crate::domain_computation::artifact_owner::WorthQueryArtifactProviderDestructorDisposition::Panicked
    );
}

#[test]
fn surviving_borrow_delays_and_then_contains_both_artifact_release_panics() {
    let world = double_panicking_artifact_world("artifact-delayed-double-panic");
    let borrowed = world
        .handle
        .borrow("delayed double-panic release")
        .expect("installed artifact contract should admit the surviving borrow");
    let terminal = world
        .running
        .completed()
        .expect("surviving artifact borrow does not prevent semantic completion");
    let pending = match terminal.cleanup() {
        WorthQueryWorkflowRunCleanupOutcome::Pending(pending) => pending,
        WorthQueryWorkflowRunCleanupOutcome::Complete(_) => {
            panic!("surviving artifact borrow allowed cleanup completion")
        }
        WorthQueryWorkflowRunCleanupOutcome::RecoveryRequired(failure) => {
            panic!("physical release ran before the last borrow left: {failure:?}")
        }
    };
    assert_eq!(world.disposal_attempts.load(Ordering::Acquire), 0);
    assert_eq!(world.destructor_attempts.load(Ordering::Acquire), 0);

    drop(borrowed);
    assert_eq!(world.disposal_attempts.load(Ordering::Acquire), 1);
    assert_eq!(world.destructor_attempts.load(Ordering::Acquire), 1);
    let release = match world.handle.owner_snapshot().provider_release() {
        crate::domain_computation::artifact_owner::WorthQueryArtifactProviderReleasePosture::RecoveryRequired(
            evidence,
        ) => evidence,
        posture => panic!("delayed double-panic artifact reported {posture:?}"),
    };
    assert_eq!(
        release.disposal(),
        crate::domain_computation::artifact_owner::WorthQueryArtifactProviderDisposalDisposition::Panicked
    );
    assert_eq!(
        release.destructor(),
        crate::domain_computation::artifact_owner::WorthQueryArtifactProviderDestructorDisposition::Panicked
    );
    drop(world.handle);
    let cleanup = match pending.retry() {
        WorthQueryWorkflowRunCleanupOutcome::Complete(cleanup) => cleanup,
        WorthQueryWorkflowRunCleanupOutcome::Pending(_) => {
            panic!("released artifact owner kept cleanup pending")
        }
        WorthQueryWorkflowRunCleanupOutcome::RecoveryRequired(failure) => {
            panic!("contained artifact panic became lower cleanup failure: {failure:?}")
        }
    };
    assert_eq!(
        cleanup.inspection().disposition(),
        WorthQueryManagedRunCleanupDisposition::RecoveryRequired
    );
    assert_eq!(
        cleanup
            .inspection()
            .artifact_evidence()
            .provider_release_recovery_required_count(),
        1
    );
}

#[test]
fn yielded_cleanup_maps_double_artifact_release_panic_into_recovery_evidence() {
    crate::domain_computation::primary_graph::with_test_advancement(|active_phase| {
        let phase = &active_phase;
        let execution = phase;

        let world = double_panicking_yield_world("yielded-artifact-double-panic");
        let active = world
            .running
            .begin_stage_graph_execution(
                execution,
                "producer",
                &world.graph,
                WorthQueryManagedGraphCallRequest::new(
                    WorthQueryGraphProviderCallKind::Observe,
                    "yielded-artifact-double-panic",
                ),
            )
            .expect("double-panic yield provider should begin");
        let paused = match active.advance(execution) {
            WorthQueryWorkflowGraphStepOutcome::Continue(paused) => paused,
            _ => panic!("double-panic yield provider did not expose its safe point"),
        };
        let yielded = match paused.yield_run() {
            crate::domain_computation::WorthQueryWorkflowYieldOutcome::Yielded(yielded) => yielded,
            _ => panic!("retained artifact prevented an otherwise eligible workflow yield"),
        };
        let cleanup = match yielded.cleanup() {
            crate::domain_computation::WorthQueryWorkflowYieldCleanupOutcome::RecoveryRequired(
                cleanup,
            ) => cleanup,
            crate::domain_computation::WorthQueryWorkflowYieldCleanupOutcome::Complete(_) => {
                panic!("double artifact release panic was reported as complete")
            }
            crate::domain_computation::WorthQueryWorkflowYieldCleanupOutcome::Pending(_) => {
                panic!("artifact without a surviving borrow remained pending")
            }
        };
        assert_eq!(
            cleanup
                .inspection()
                .artifact_evidence()
                .provider_release_recovery_required_count(),
            1
        );
        assert_eq!(world.disposal_attempts.load(Ordering::Acquire), 1);
        assert_eq!(world.destructor_attempts.load(Ordering::Acquire), 1);
        assert!(cleanup.inspection().resources_released());
        assert_eq!(cleanup.inspection().released_reservation_count(), 3);
        let release = match world.handle.owner_snapshot().provider_release() {
        crate::domain_computation::artifact_owner::WorthQueryArtifactProviderReleasePosture::RecoveryRequired(
            evidence,
        ) => evidence,
        posture => panic!("yielded double-panic artifact reported {posture:?}"),
    };
        assert_eq!(
        release.disposal(),
        crate::domain_computation::artifact_owner::WorthQueryArtifactProviderDisposalDisposition::Panicked
    );
        assert_eq!(
        release.destructor(),
        crate::domain_computation::artifact_owner::WorthQueryArtifactProviderDestructorDisposition::Panicked
    );
    });
}

struct DoublePanickingArtifactWorld {
    running: crate::domain_computation::WorthQueryRunningWorkflowRun,
    handle: crate::domain_computation::WorthQueryMoveOnlyArtifactHandle,
    disposal_attempts: Arc<AtomicUsize>,
    destructor_attempts: Arc<AtomicUsize>,
}

struct DoublePanickingYieldWorld {
    running: crate::domain_computation::WorthQueryRunningWorkflowRun,
    graph: WorthQueryInstalledGraphParticipationAuthority,
    handle: crate::domain_computation::WorthQueryMoveOnlyArtifactHandle,
    disposal_attempts: Arc<AtomicUsize>,
    destructor_attempts: Arc<AtomicUsize>,
}

fn double_panicking_artifact_world(label: &str) -> DoublePanickingArtifactWorld {
    let runtime = query_runtime();
    let operation_resources = admitted_plan(label, 8);
    let stage_label = format!("{label}:producer");
    let stage_resources = admitted_plan(&stage_label, 4);
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
        .expect("double-panic workflow should reserve");
    let lower = causal_fixture::managed_admission_context();
    let running = runtime
        .managed_run_admission(&lower.bridge, &lower.relational)
        .admit_workflow(&operation, attempt, lower.read_request())
        .expect("double-panic workflow should admit")
        .start()
        .expect("double-panic workflow should start");
    let production = running
        .artifacts()
        .production_authority("producer")
        .expect("producer stage should validate")
        .expect("producer stage should own output authority");
    let admission =
        crate::domain_computation::artifact_owner::WorthQueryArtifactProductionAuthority::admit(
            &production,
            WorthQueryArtifactProductionEvidence::new(
                "double-panic-provenance",
                "double-panic-dependency",
            ),
        );
    let disposal_attempts = Arc::new(AtomicUsize::new(0));
    let destructor_attempts = Arc::new(AtomicUsize::new(0));
    let handle =
        crate::domain_computation::artifact_owner::WorthQueryArtifactProductionAuthority::register_exact(
            &production,
            admission,
            DoublePanickingArtifactResource {
                disposal_attempts: Arc::clone(&disposal_attempts),
                destructor_attempts: Arc::clone(&destructor_attempts),
            },
        )
        .expect("double-panic artifact should register before production freezes");
    DoublePanickingArtifactWorld {
        running,
        handle,
        disposal_attempts,
        destructor_attempts,
    }
}

fn double_panicking_yield_world(label: &str) -> DoublePanickingYieldWorld {
    let installer = WorthQueryExecutionRuntimeInstaller::new();
    let provider_anchor = Arc::new(
        crate::domain_computation::provider_session::graph_provider::bounded_step::provider_anchor::WorthQueryGraphProviderAnchor::install::<ManagedGraph, _>(
            super::yield_fixture::YieldProvider::installed(7),
        ),
    );
    let provider_support = provider_anchor.resource_support().clone();
    let graph = super::workflow_provider_steps::installed_graph(
        &installer,
        &format!("{label}-graph"),
        provider_anchor,
    );
    let runtime =
        super::workflow_provider_steps::installed_runtime(installer, "double-panic yield cleanup");
    let operation_resources =
        crate::domain_computation::provider_session::admitted_yield_plan(label, 8);
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
        crate::domain_computation::artifact_owner::installed_artifact_contract_for_managed_run();
    let operation = workflow_authority_with_stage_graph_and_output_artifact(
        &runtime,
        &resources,
        "producer",
        &graph,
        WorthQueryOperationGraphAccess::Observe,
        output,
    );
    let running =
        super::workflow_provider_steps::admitted_workflow(&runtime, &operation, resources);
    let production = running
        .artifacts()
        .production_authority("producer")
        .expect("producer stage should validate")
        .expect("producer stage should own output authority");
    let admission =
        crate::domain_computation::artifact_owner::WorthQueryArtifactProductionAuthority::admit(
            &production,
            WorthQueryArtifactProductionEvidence::new(
                "yielded-double-panic-provenance",
                "yielded-double-panic-dependency",
            ),
        );
    let disposal_attempts = Arc::new(AtomicUsize::new(0));
    let destructor_attempts = Arc::new(AtomicUsize::new(0));
    let handle =
        crate::domain_computation::artifact_owner::WorthQueryArtifactProductionAuthority::register_exact(
            &production,
            admission,
            DoublePanickingArtifactResource {
                disposal_attempts: Arc::clone(&disposal_attempts),
                destructor_attempts: Arc::clone(&destructor_attempts),
            },
        )
        .expect("double-panic artifact should register before yield freezes production");
    DoublePanickingYieldWorld {
        running,
        graph,
        handle,
        disposal_attempts,
        destructor_attempts,
    }
}

struct DoublePanickingArtifactResource {
    disposal_attempts: Arc<AtomicUsize>,
    destructor_attempts: Arc<AtomicUsize>,
}

impl WorthQueryArtifactProviderResource for DoublePanickingArtifactResource {
    const PROVIDER_FAMILY: &'static str = "WORTH.tests.affinity.provider";

    fn canonical_semantic_projection(&self) -> Vec<u8> {
        b"double-panic-artifact".to_vec()
    }

    fn retained_bytes(&self) -> usize {
        64
    }

    fn dispose(&mut self) {
        self.disposal_attempts.fetch_add(1, Ordering::AcqRel);
        panic!("artifact provider disposal panicked")
    }
}

impl Drop for DoublePanickingArtifactResource {
    fn drop(&mut self) {
        self.destructor_attempts.fetch_add(1, Ordering::AcqRel);
        panic!("artifact provider destructor panicked")
    }
}
