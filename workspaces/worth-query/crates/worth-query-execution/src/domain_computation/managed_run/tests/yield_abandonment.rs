use super::yield_fixture::YieldProvider;
use super::*;

#[test]
fn dropping_yielded_workflow_closes_artifacts_despite_surviving_production_authority() {
    crate::domain_computation::primary_graph::with_test_advancement(|active_phase| {
        let phase = &active_phase;
        let execution = phase;

        let installer = WorthQueryExecutionRuntimeInstaller::new();
        let provider_anchor = Arc::new(
        crate::domain_computation::provider_session::graph_provider::bounded_step::provider_anchor::WorthQueryGraphProviderAnchor::install::<ManagedGraph, _>(
            YieldProvider::installed(5),
        ),
    );
        let provider_support = provider_anchor.resource_support().clone();
        let graph = super::workflow_provider_steps::installed_graph(
            &installer,
            "yield-abandonment-graph",
            provider_anchor,
        );
        let runtime =
            super::workflow_provider_steps::installed_runtime(installer, "yield abandonment");
        let operation_resources = crate::domain_computation::provider_session::admitted_yield_plan(
            "yield-abandonment",
            8,
        );
        let stage_resources = admitted_plan_with_graph_support(
            "yield-abandonment:producer",
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
        let running =
            super::workflow_provider_steps::admitted_workflow(&runtime, &operation, resources);
        let production = running
            .artifacts()
            .production_authority("producer")
            .unwrap()
            .expect("producer output artifact should install");
        let disposals = Arc::new(AtomicUsize::new(0));
        let handle = register_abandonment_artifact(&production, "retained", Arc::clone(&disposals))
            .expect("pre-yield artifact should register");
        let active = running
            .begin_stage_graph_execution(
                execution,
                "producer",
                &graph,
                WorthQueryManagedGraphCallRequest::new(
                    WorthQueryGraphProviderCallKind::Observe,
                    "yield-abandonment",
                ),
            )
            .unwrap();
        let paused = match active.advance(execution) {
            WorthQueryWorkflowGraphStepOutcome::Continue(paused) => paused,
            _ => panic!("yield abandonment provider did not pause"),
        };
        let yielded = match paused.yield_run() {
            crate::domain_computation::WorthQueryWorkflowYieldOutcome::Yielded(yielded) => yielded,
            _ => panic!("eligible workflow did not yield"),
        };
        assert_eq!(disposals.load(Ordering::Acquire), 0);

        drop(yielded);

        assert_eq!(disposals.load(Ordering::Acquire), 1);
        assert!(handle.owner_snapshot().is_disposed());
        let rejected_disposals = Arc::new(AtomicUsize::new(0));
        let denial = register_abandonment_artifact(
            &production,
            "post-abandonment",
            Arc::clone(&rejected_disposals),
        )
        .expect_err("abandoned yielded workflow retained production authority");
        assert_eq!(
            denial.kind(),
            crate::domain_computation::WorthQueryArtifactDenialKind::AlreadyDisposed
        );
        assert_eq!(rejected_disposals.load(Ordering::Acquire), 1);
        drop(handle);
        assert_eq!(disposals.load(Ordering::Acquire), 1);
    });
}

fn register_abandonment_artifact(
    production: &Arc<
        crate::domain_computation::artifact_owner::WorthQueryArtifactProductionAuthority,
    >,
    label: &str,
    disposals: Arc<AtomicUsize>,
) -> Result<
    crate::domain_computation::WorthQueryMoveOnlyArtifactHandle,
    crate::domain_computation::WorthQueryArtifactDenial,
> {
    let admission =
        crate::domain_computation::artifact_owner::WorthQueryArtifactProductionAuthority::admit(
            production,
            WorthQueryArtifactProductionEvidence::new(
                format!("{label}-provenance"),
                format!("{label}-dependency"),
            ),
        );
    crate::domain_computation::artifact_owner::WorthQueryArtifactProductionAuthority::register_exact(
        production,
        admission,
        AbandonmentArtifact(disposals),
    )
}

#[derive(Debug)]
struct AbandonmentArtifact(Arc<AtomicUsize>);

impl WorthQueryArtifactProviderResource for AbandonmentArtifact {
    const PROVIDER_FAMILY: &'static str = "WORTH.tests.affinity.provider";

    fn canonical_semantic_projection(&self) -> Vec<u8> {
        b"yield-abandonment-artifact".to_vec()
    }

    fn retained_bytes(&self) -> usize {
        48
    }

    fn dispose(&mut self) {
        self.0.fetch_add(1, Ordering::AcqRel);
    }
}
