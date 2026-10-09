use std::sync::Mutex;

use super::*;

type ProductionAuthority =
    Arc<crate::domain_computation::artifact_owner::WorthQueryArtifactProductionAuthority>;

struct FreezeProbeProvider {
    authority: Arc<Mutex<Option<ProductionAuthority>>>,
    result: Arc<Mutex<Option<FreezeProbeResult>>>,
    disposals: Arc<AtomicUsize>,
}

struct FreezeProbeExecution {
    authority: Arc<Mutex<Option<ProductionAuthority>>>,
    result: Arc<Mutex<Option<FreezeProbeResult>>>,
    disposals: Arc<AtomicUsize>,
    retained: Option<WorthQueryGraphProviderRetainedMemory>,
}

enum FreezeProbeResult {
    Registered,
    Denied(crate::domain_computation::WorthQueryArtifactDenial),
}

impl WorthQueryGraphParticipationProvider<ManagedGraph> for FreezeProbeProvider {
    type Execution = FreezeProbeExecution;

    fn execution_resource_support(
        &self,
    ) -> worth_query_admission::facade::resource_admission::WorthQueryExecutionResourceSupport {
        crate::domain_computation::provider_session::execution_resource_support_with_yield(
            "yield-production-freeze-provider",
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
            FreezeProbeExecution {
                authority: Arc::clone(&self.authority),
                result: Arc::clone(&self.result),
                disposals: Arc::clone(&self.disposals),
                retained: None,
            },
        )
    }
}

impl WorthQueryGraphProviderExecution for FreezeProbeExecution {
    fn advance(
        &mut self,
        step: &mut WorthQueryGraphProviderStep,
    ) -> Result<WorthQueryGraphProviderStepDisposition, WorthQueryGraphProviderFailure> {
        step.perform_work_unit(|| Ok(()))?;
        self.retained = Some(step.retain_bytes(3).map_err(step_failure)?);
        step.record_checkpoint_available().map_err(step_failure)?;
        Ok(WorthQueryGraphProviderStepDisposition::continue_work())
    }

    fn suspend(
        &mut self,
    ) -> Result<
        Box<dyn crate::domain_computation::WorthQueryGraphProviderCheckpoint>,
        WorthQueryGraphProviderFailure,
    > {
        let authority = self
            .authority
            .lock()
            .expect("freeze probe authority lock remains available")
            .clone()
            .expect("workflow installs production authority before provider suspension");
        let admission =
            crate::domain_computation::artifact_owner::WorthQueryArtifactProductionAuthority::admit(
                &authority,
                WorthQueryArtifactProductionEvidence::new(
                    "suspension-race-provenance",
                    "suspension-race-dependency",
                ),
            );
        let result =
            crate::domain_computation::artifact_owner::WorthQueryArtifactProductionAuthority::register_exact(
                &authority,
                admission,
                FreezeProbeArtifact(Arc::clone(&self.disposals)),
            );
        *self
            .result
            .lock()
            .expect("freeze probe result lock remains available") = Some(match result {
            Ok(handle) => {
                drop(handle);
                FreezeProbeResult::Registered
            }
            Err(denial) => FreezeProbeResult::Denied(denial),
        });
        Ok(Box::new(FreezeProbeCheckpoint {
            _retained: self
                .retained
                .take()
                .expect("freeze checkpoint transfers governed retained memory once"),
        }))
    }

    fn dispose(&mut self) -> Result<(), WorthQueryGraphProviderFailure> {
        Ok(())
    }
}

struct FreezeProbeCheckpoint {
    _retained: WorthQueryGraphProviderRetainedMemory,
}

impl crate::domain_computation::WorthQueryGraphProviderCheckpoint for FreezeProbeCheckpoint {
    fn retained_bytes(&self) -> u64 {
        3
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
            "Phase 6.3 freeze probe must never restore",
        ))
    }
}

struct FreezeProbeArtifact(Arc<AtomicUsize>);

impl WorthQueryArtifactProviderResource for FreezeProbeArtifact {
    const PROVIDER_FAMILY: &'static str = "WORTH.tests.affinity.provider";

    fn canonical_semantic_projection(&self) -> Vec<u8> {
        b"yield-freeze-probe".to_vec()
    }

    fn retained_bytes(&self) -> usize {
        8
    }

    fn dispose(&mut self) {
        self.0.fetch_add(1, Ordering::AcqRel);
    }
}

#[test]
fn workflow_freezes_artifact_production_before_provider_suspension() {
    crate::domain_computation::primary_graph::with_test_advancement(|active_phase| {
        let phase = &active_phase;
        let execution = phase;

        let authority = Arc::new(Mutex::new(None));
        let result = Arc::new(Mutex::new(None));
        let disposals = Arc::new(AtomicUsize::new(0));
        let provider = FreezeProbeProvider {
            authority: Arc::clone(&authority),
            result: Arc::clone(&result),
            disposals: Arc::clone(&disposals),
        };
        let (running, graph, production) = freeze_probe_workflow(provider);
        *authority
            .lock()
            .expect("freeze probe authority lock remains available") = Some(production);
        let active = running
            .begin_stage_graph_execution(
                execution,
                "producer",
                &graph,
                WorthQueryManagedGraphCallRequest::new(
                    WorthQueryGraphProviderCallKind::Observe,
                    "yield-production-freeze",
                ),
            )
            .expect("freeze probe provider should begin");
        let paused = match active.advance(execution) {
            WorthQueryWorkflowGraphStepOutcome::Continue(paused) => paused,
            _ => panic!("freeze probe provider did not reach its safe point"),
        };
        let yielded = match paused.yield_run() {
            crate::domain_computation::WorthQueryWorkflowYieldOutcome::Yielded(yielded) => yielded,
            _ => panic!("rejected suspension-time production prevented an eligible yield"),
        };
        let denial = match result
            .lock()
            .expect("freeze probe result lock remains available")
            .take()
            .expect("provider suspension must attempt artifact production")
        {
            FreezeProbeResult::Denied(denial) => denial,
            FreezeProbeResult::Registered => {
                panic!("provider registered an artifact after workflow yield began")
            }
        };
        assert_eq!(
            denial.kind(),
            crate::domain_computation::WorthQueryArtifactDenialKind::ProductionClosed
        );
        let release = match denial.rejected_resource_release() {
        Some(crate::domain_computation::artifact_owner::WorthQueryArtifactProviderReleasePosture::Complete(
            evidence,
        )) => evidence,
        posture => panic!("rejected suspension-time artifact reported {posture:?}"),
    };
        assert_eq!(
        release.disposal(),
        crate::domain_computation::artifact_owner::WorthQueryArtifactProviderDisposalDisposition::Completed
    );
        assert_eq!(
        release.destructor(),
        crate::domain_computation::artifact_owner::WorthQueryArtifactProviderDestructorDisposition::Completed
    );
        assert_eq!(disposals.load(Ordering::Acquire), 1);
        assert_eq!(
            yielded
                .inspection()
                .artifact_evidence()
                .produced_artifact_count(),
            0
        );
        assert_eq!(yielded.inspection().artifact_evidence().retained_bytes(), 0);
        match yielded.cleanup() {
            crate::domain_computation::WorthQueryWorkflowYieldCleanupOutcome::Complete(_) => {}
            _ => panic!("artifact-free freeze probe did not clean up"),
        }
    });
}

fn freeze_probe_workflow(
    provider: FreezeProbeProvider,
) -> (
    crate::domain_computation::WorthQueryRunningWorkflowRun,
    WorthQueryInstalledGraphParticipationAuthority,
    ProductionAuthority,
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
        "yield-production-freeze-graph",
        provider_anchor,
    );
    let runtime =
        super::workflow_provider_steps::installed_runtime(installer, "yield production freeze");
    let operation_resources = crate::domain_computation::provider_session::admitted_yield_plan(
        "yield-production-freeze",
        8,
    );
    let stage_resources = admitted_plan_with_graph_support(
        "yield-production-freeze:producer",
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
        .expect("producer role should validate")
        .expect("producer output contract should install");
    (running, graph, production)
}

fn step_failure(
    denial: crate::domain_computation::WorthQueryGraphProviderStepDenial,
) -> WorthQueryGraphProviderFailure {
    WorthQueryGraphProviderFailure::new(denial.detail())
}
