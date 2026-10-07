use serde::Serialize;
use sha2::{Digest, Sha256};

use crate::facade::*;
use crate::tests::async_node_support::{
    async_node_capability_declaration, AsyncNodeTestRuntime as TestRuntime,
};

#[derive(Serialize)]
struct ReplayRestoreDigestBasis {
    replay_digest: String,
    branch_restore_report: Option<ResourceBranchRestoreReport>,
}

struct NoopAsyncNodeObservationListener;

impl ObservationListener<(), (), (), (), ()> for NoopAsyncNodeObservationListener {
    fn on_observation(
        &self,
        _ctx: ObservationReadContext<'_, (), (), (), (), ()>,
        _notice: &ObservationNotice<'_>,
    ) {
    }
}

#[test]
fn async_node_capability_equivalence_report_matches_legacy_runtime_truth_for_rich_leaf_workload() {
    let mut graph = SignalGraph::new();
    let node = graph.node().build();
    let mut runtime = TestRuntime::build(graph);
    let declaration = async_node_capability_declaration(node)
        .with_observation_policy(ResourceObservationPolicyDeclaration::LifecycleAndOutput)
        .with_output_continuity_policy(ResourceOutputContinuityPolicyDeclaration::HideWhilePending);
    let attached = runtime
        .attach_async_capability(declaration.clone())
        .expect("async capability should attach");

    runtime.observe_nodes(
        ObservationPolicy::visited(),
        [node],
        Box::new(NoopAsyncNodeObservationListener),
    );
    let admitted_request = runtime
        .admit_async_node_request(attached.request_intent())
        .expect("request should admit through attached capability handle")
        .resource_admission()
        .expect("request admission should lower into resource admission")
        .admitted_request();
    let admitted_completion = runtime
        .admit_resource_completion(RawCompletionEnvelope::new(
            admitted_request.handle().request_id(),
            admitted_request.handle().generation(),
            admitted_request.handle().branch_epoch(),
            admitted_request.attempt(),
            attached.payload_contract_digest().clone(),
            64,
        ))
        .admitted_completion()
        .expect("completion should be admitted");
    let mut ctx = ();
    runtime
        .transaction(&mut ctx, |tx| {
            let staged = tx.stage_admitted_resource_completion(admitted_completion)?;
            tx.commit_staged_resource_completion(staged.staged_effect())?;
            Ok(())
        })
        .expect("completion should commit and materialize observation lineage");

    let budget = ResourceDiagnosticsExpansionBudget::allow_cold_reconstruction(u32::MAX);
    let report = runtime
        .async_node_capability_equivalence_report(&attached, &declaration, budget)
        .expect("equivalence report should materialize for current capability handle");
    let direct_replay = runtime.reconstruct_resource_replay_summary();
    let direct_observation = runtime
        .latest_resource_observation_batch_report()
        .expect("rich equivalence path should have direct observation lineage");
    let (direct_explanation_artifact, direct_explanation_availability) = runtime
        .graph()
        .materialize_explanation_artifact(node)
        .expect("direct explanation materialization should succeed");
    let expected_observation_digest = canonical_digest(&direct_observation);
    let expected_explanation_digest = direct_explanation_artifact.as_ref().map(|artifact| {
        canonical_digest(&artifact.diagnostics_summary(runtime.runtime_policy().tier))
    });

    assert_eq!(report.node(), node);
    assert_eq!(
        report.capability_declaration_digest(),
        canonical_digest(&declaration)
    );
    assert_eq!(
        report.legacy_declaration_digest(),
        canonical_digest(&declaration.clone().into_legacy_resource_declaration())
    );
    assert_eq!(
        report.registry_digest().as_str(),
        report
            .alias_lowering_proof()
            .legacy_registry_digest()
            .as_str()
    );
    assert_eq!(
        report.bundle_digest().as_str(),
        report
            .alias_lowering_proof()
            .legacy_bundle_digest()
            .as_str()
    );
    assert_eq!(
        report.payload_contract_digest().as_str(),
        report
            .alias_lowering_proof()
            .legacy_payload_contract_digest()
            .as_str()
    );
    assert_eq!(report.lifecycle_digest(), direct_replay.lifecycle_digest());
    assert_eq!(
        report.output_continuity_digest(),
        direct_replay.output_continuity_digest()
    );
    assert_eq!(
        report.denial_digest(),
        direct_replay.denied_completion_digest()
    );
    assert_eq!(
        report.observation_digest(),
        Some(expected_observation_digest.as_str())
    );
    assert_eq!(
        report.explanation_digest(),
        expected_explanation_digest.as_deref()
    );
    assert_eq!(
        report.explanation_availability(),
        direct_explanation_availability
    );
    assert_eq!(
        report.replay_restore_digest(),
        canonical_digest(&ReplayRestoreDigestBasis {
            replay_digest: direct_replay.replay_digest().to_owned(),
            branch_restore_report: None,
        })
    );
    assert_eq!(
        report.performance().boundary(),
        ResourceBoundaryKind::AsyncNodeCapabilityEquivalence
    );
    assert_eq!(
        runtime
            .telemetry()
            .resource
            .async_node_capability_equivalence_count,
        1
    );
}

#[test]
fn async_node_capability_equivalence_report_denies_stale_handle_after_restore() {
    let mut graph = SignalGraph::new();
    let node = graph.node().build();
    let mut runtime = TestRuntime::build(graph);
    let snapshot = runtime
        .capture_snapshot()
        .expect("snapshot capture should succeed without managed queue bindings");
    let declaration = async_node_capability_declaration(node);
    let attached = runtime
        .attach_async_capability(declaration.clone())
        .expect("async capability should attach after snapshot");

    runtime
        .restore_snapshot(&snapshot)
        .expect("restore should erase post-snapshot capability attachment");

    let denial = runtime
        .async_node_capability_equivalence_report(
            &attached,
            &declaration,
            ResourceDiagnosticsExpansionBudget::allow_cold_reconstruction(u32::MAX),
        )
        .expect_err("restored-away capability handle should deny equivalence");

    assert_eq!(
        denial.denial_class(),
        AsyncNodeCapabilityEquivalenceDenialClass::HistoricalParityDenied
    );
    assert_eq!(denial.handle_node(), node);
    assert_eq!(denial.declaration_node(), node);
    assert_eq!(
        denial
            .historical_parity_denial()
            .expect("historical parity denial should be preserved")
            .denial_class(),
        AsyncNodeHistoricalParityDenialClass::UndeclaredCapability
    );
    assert_eq!(
        denial.performance().boundary(),
        ResourceBoundaryKind::AsyncNodeCapabilityEquivalence
    );
}

#[test]
fn async_node_capability_equivalence_report_rejects_mismatched_declaration_for_same_node() {
    let mut graph = SignalGraph::new();
    let node = graph.node().build();
    let mut runtime = TestRuntime::build(graph);
    let attached = runtime
        .attach_async_capability(async_node_capability_declaration(node))
        .expect("async capability should attach");
    let mismatched_declaration = AsyncNodeCapabilityDeclaration::new(
        node,
        AsyncNodePayloadContract::new(AsyncNodePayloadContractId::new(99))
            .with_max_payload_bytes(2048),
    );

    let denial = runtime
        .async_node_capability_equivalence_report(
            &attached,
            &mismatched_declaration,
            ResourceDiagnosticsExpansionBudget::allow_cold_reconstruction(u32::MAX),
        )
        .expect_err("same-node declaration drift must deny equivalence");

    assert_eq!(
        denial.denial_class(),
        AsyncNodeCapabilityEquivalenceDenialClass::HandleDeclarationDigestMismatch
    );
    assert_eq!(denial.handle_node(), node);
    assert_eq!(denial.declaration_node(), node);
    assert!(denial.historical_parity_denial().is_none());
    assert_eq!(
        denial.performance().boundary(),
        ResourceBoundaryKind::AsyncNodeCapabilityEquivalence
    );
}

#[test]
fn async_node_capability_equivalence_report_carries_explanation_denial_without_losing_replay_truth()
{
    let mut graph = SignalGraph::new();
    let node = graph.node().build();
    let mut runtime = TestRuntime::build(graph);
    let declaration = async_node_capability_declaration(node);
    let attached = runtime
        .attach_async_capability(declaration.clone())
        .expect("async capability should attach");

    runtime
        .admit_async_node_request(attached.request_intent())
        .expect("request should admit through attached capability handle");
    runtime.adjust_runtime_policy(|mut policy| {
        policy
            .reconstruction_budget
            .allow_explanation_reconstruction = false;
        policy
    });

    let budget = ResourceDiagnosticsExpansionBudget::retained_summary_only();
    let report = runtime
        .async_node_capability_equivalence_report(&attached, &declaration, budget)
        .expect("equivalence report should still materialize when explanation richness denies");
    let direct_replay = runtime.reconstruct_resource_replay_summary();

    assert_eq!(
        report.performance().boundary(),
        ResourceBoundaryKind::AsyncNodeCapabilityEquivalence
    );
    assert_eq!(
        report.explanation_availability(),
        DiagnosticsAvailability::DeniedByBudget
    );
    assert!(report.explanation_digest().is_none());
    assert!(report
        .historical_parity_report()
        .explanation_summary()
        .is_none());
    assert_eq!(
        report.replay_restore_digest(),
        canonical_digest(&ReplayRestoreDigestBasis {
            replay_digest: direct_replay.replay_digest().to_owned(),
            branch_restore_report: None,
        })
    );
}

pub(crate) fn canonical_digest<T: Serialize>(value: &T) -> String {
    let bytes = serde_json::to_vec(value).expect("test digest serialization should succeed");
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}
