use std::collections::BTreeMap;
use std::sync::Arc;

use super::yield_fixture::YieldProvider;
use super::*;

#[test]
fn over_ceiling_workflow_artifacts_deny_yield_without_consuming_the_run() {
    crate::domain_computation::primary_graph::with_test_advancement(|active_phase| {
        let bootstrap = active_phase.bootstrap_for_test();
        let resource_request = bootstrap.execution_request();

        let phase = &active_phase;
        let execution = phase;

        let installer = WorthQueryExecutionRuntimeInstaller::new();
        let provider_anchor = Arc::new(
        crate::domain_computation::provider_session::graph_provider::bounded_step::provider_anchor::WorthQueryGraphProviderAnchor::install::<ManagedGraph, _>(
            YieldProvider::installed(7),
        ),
    );
        let provider_support = provider_anchor.resource_support().clone();
        let graph = super::workflow_provider_steps::installed_graph(
            &installer,
            "workflow-yield-ceiling-graph",
            provider_anchor,
        );
        let runtime =
            super::workflow_provider_steps::installed_runtime(installer, "workflow yield ceiling");
        let operation_resources = crate::domain_computation::provider_session::admitted_yield_plan(
            "workflow-yield-ceiling",
            8,
        );
        let stage_resources = admitted_plan_with_graph_support(
            "workflow-yield-ceiling:producer",
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
        let production = running
            .artifacts()
            .production_authority("producer")
            .unwrap()
            .expect("producer output should install");
        let admission =
            crate::domain_computation::artifact_owner::WorthQueryArtifactProductionAuthority::admit(
                &production,
                WorthQueryArtifactProductionEvidence::new(
                    "over-ceiling-provenance",
                    "over-ceiling-dependency",
                ),
            );
        let handle =
        crate::domain_computation::artifact_owner::WorthQueryArtifactProductionAuthority::register_exact(
            &production,
            admission,
            OverCeilingArtifact,
        )
        .expect("exact artifact owner should register");
        let active = running
            .begin_stage_graph_execution(
                execution,
                "producer",
                &graph,
                WorthQueryManagedGraphCallRequest::new(
                    WorthQueryGraphProviderCallKind::Observe,
                    "workflow-yield-ceiling",
                ),
            )
            .unwrap();
        let paused = match active.advance(execution) {
            WorthQueryWorkflowGraphStepOutcome::Continue(paused) => paused,
            _ => panic!("workflow provider did not pause"),
        };
        let denied = match paused.yield_run() {
            crate::domain_computation::WorthQueryWorkflowYieldOutcome::Denied(denied) => denied,
            _ => panic!("over-ceiling workflow artifacts did not deny before transition"),
        };
        assert_eq!(
            denied.kind(),
            crate::domain_computation::WorthQueryWorkflowYieldDenialKind::RetainedBytesExceeded
        );
        let resumed_admission =
            crate::domain_computation::artifact_owner::WorthQueryArtifactProductionAuthority::admit(
                &production,
                WorthQueryArtifactProductionEvidence::new(
                    "post-denial-provenance",
                    "post-denial-dependency",
                ),
            );
        let post_denial_handle =
        crate::domain_computation::artifact_owner::WorthQueryArtifactProductionAuthority::register_exact(
            &production,
            resumed_admission,
            OverCeilingArtifact,
        )
        .expect("denied yield must restore the active artifact production generation");
        drop(post_denial_handle);
        drop(handle);
        let completion = match denied.into_paused().advance(execution) {
            WorthQueryWorkflowGraphStepOutcome::Completed(completion) => completion,
            _ => panic!("retained-memory denial consumed the paused workflow"),
        };
        let terminal = completion.into_running().completed().unwrap();
        match terminal.cleanup() {
            WorthQueryWorkflowRunCleanupOutcome::Complete(_) => {}
            _ => panic!("released over-ceiling artifact prevented workflow cleanup"),
        }
    });
}

struct OverCeilingArtifact;

impl WorthQueryArtifactProviderResource for OverCeilingArtifact {
    const PROVIDER_FAMILY: &'static str = "WORTH.tests.affinity.provider";

    fn canonical_semantic_projection(&self) -> Vec<u8> {
        b"over-ceiling-yield-artifact".to_vec()
    }

    fn retained_bytes(&self) -> usize {
        4_096
    }

    fn dispose(&mut self) {}
}
