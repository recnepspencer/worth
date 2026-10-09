use crate::data::dependency::DependencySnapshot;
use crate::data::trace::CausalityMetadata;
use crate::data::trace::RetainedDiagnosticArtifact;
use crate::diagnostics::ExplanationFact;
use crate::diagnostics::ProvenanceFact;
use crate::facade::{
    mark_dirty, ArtifactRetentionPolicy, CanonicalChangedRegions, ChangedRegion,
    DiagnosticsAvailability, NodeEvaluationResult, SignalGraph, SignalRuntime, SignalRuntimePolicy,
    SnapshotRestoreIntent,
};
use crate::tests::support::{evaluate, version_ab, GraphDependencyBatchExt, ASPECT_A};

#[test]
fn checkpoint_image_strips_node_local_cold_payloads_while_snapshot_bundle_retains_them() {
    let mut graph = SignalGraph::new();
    graph.set_runtime_policy(SignalRuntimePolicy::development());
    let node = graph.node().output_identity().build();

    evaluate(&mut graph, node, &mut |_id, _graph| {
        Ok(NodeEvaluationResult::from_version(version_ab(1, 0))
            .with_output_identity("checkpoint-cold"))
    })
    .unwrap();

    {
        let mut entry = graph.get_entry_mut(node).unwrap();
        entry.set_retained_diagnostic_artifact(Some(RetainedDiagnosticArtifact {
            changed_regions: CanonicalChangedRegions::from(vec![ChangedRegion::new("wing")]),
            labels: vec!["retained".to_string()],
            keyed_family: Some("airframe".to_string()),
            keyed_key: Some("wing".to_string()),
            reuse_certification: None,
            reuse_boundary_context: None,
        }));
        entry.set_causality(Some(CausalityMetadata {
            kind: "bridge".to_string(),
            fields: [("patch".to_string(), "s9-12".to_string())]
                .into_iter()
                .collect(),
        }));
    }

    let snapshot = graph.capture_snapshot();

    let checkpoint_graph = snapshot.authority_graph().unwrap();
    let checkpoint_entry = checkpoint_graph
        .get_entry(node)
        .expect("checkpoint node entry");
    assert!(
        checkpoint_entry.retained_diagnostic_artifact().is_none(),
        "checkpoint image must not carry retained node-local cold artifacts"
    );
    assert!(
        checkpoint_entry.get_causality().is_none(),
        "checkpoint image must not carry causality metadata through the authority lane"
    );

    let rich_entry = snapshot
        .diagnostic_graph
        .get_entry(node)
        .expect("rich snapshot node entry");
    assert!(
        rich_entry.retained_diagnostic_artifact().is_some(),
        "rich snapshot bundle should still retain node-local cold artifacts for diagnostics"
    );
    assert!(
        rich_entry.get_causality().is_some(),
        "rich snapshot bundle should still retain node-local causality for diagnostics"
    );
}

#[test]
fn checkpoint_image_omits_dependency_snapshots_and_restore_rebuilds_them_from_explicit_batch() {
    let mut graph = SignalGraph::new();
    let source = graph.node().output_identity().build();
    let target = graph.node().build();
    graph.append_dependency(target, source, ASPECT_A).unwrap();

    evaluate(&mut graph, source, &mut |_id, _graph| {
        Ok(NodeEvaluationResult::from_version(version_ab(1, 0))
            .with_output_identity("checkpoint-deps"))
    })
    .unwrap();
    evaluate(&mut graph, target, &mut |_id, graph| {
        Ok(NodeEvaluationResult::from_version(
            graph.get_entry(source).unwrap().get_aspect_version(),
        ))
    })
    .unwrap();

    let snapshot = graph.capture_snapshot();
    let authority_graph =
        SignalGraph::restore_from_checkpoint_authority(&snapshot.checkpoint_image.authority)
            .unwrap();
    assert!(
        authority_graph
            .get_dep_snapshot(target)
            .unwrap()
            .entries()
            .is_empty(),
        "checkpoint authority lane must not carry dependency snapshot state"
    );
    assert_eq!(
        snapshot
            .checkpoint_image
            .dependency_snapshot_batch
            .target_nodes()
            .as_slice(),
        &[target],
        "checkpoint image should carry dependency snapshot rebuild work explicitly"
    );
    assert_eq!(
        snapshot
            .authority_graph()
            .unwrap()
            .get_dep_snapshot(target)
            .unwrap()
            .entries()[0]
            .cached_version,
        1,
        "supported authority graph reconstruction must apply the explicit rebuild batch"
    );

    let mut overwritten = DependencySnapshot::empty();
    overwritten.record(source, ASPECT_A, 9, None);
    graph.set_dep_snapshot(target, overwritten).unwrap();
    assert_eq!(
        graph.get_dep_snapshot(target).unwrap().entries()[0].cached_version,
        9
    );

    graph.restore_snapshot(&snapshot).unwrap();

    assert_eq!(
        graph.get_dep_snapshot(target).unwrap().entries()[0].cached_version,
        1,
        "restore must rebuild dependency snapshots from the explicit checkpoint batch"
    );
}

#[test]
fn restore_uses_checkpoint_authority_even_when_rich_snapshot_node_cold_payloads_are_tampered() {
    let mut graph = SignalGraph::new();
    graph.set_runtime_policy(SignalRuntimePolicy::development());
    let node = graph.node().output_identity().build();

    evaluate(&mut graph, node, &mut |_id, _graph| {
        Ok(NodeEvaluationResult::from_version(version_ab(1, 0))
            .with_output_identity("checkpoint-restore"))
    })
    .unwrap();

    {
        let mut entry = graph.get_entry_mut(node).unwrap();
        entry.set_retained_diagnostic_artifact(Some(RetainedDiagnosticArtifact {
            changed_regions: CanonicalChangedRegions::from(vec![ChangedRegion::new("fuselage")]),
            labels: vec!["captured".to_string()],
            keyed_family: Some("airframe".to_string()),
            keyed_key: Some("fuselage".to_string()),
            reuse_certification: None,
            reuse_boundary_context: None,
        }));
        entry.set_causality(Some(CausalityMetadata {
            kind: "capture".to_string(),
            fields: [("rev".to_string(), "1".to_string())].into_iter().collect(),
        }));
    }

    let snapshot = graph.capture_snapshot();

    {
        let mut entry = graph.get_entry_mut(node).unwrap();
        entry.set_retained_diagnostic_artifact(None);
        entry.set_causality(None);
    }
    mark_dirty(&mut graph, node, ASPECT_A).unwrap();
    evaluate(&mut graph, node, &mut |_id, _graph| {
        Ok(NodeEvaluationResult::from_version(version_ab(2, 0))
            .with_output_identity("checkpoint-restore-updated"))
    })
    .unwrap();

    let mut tampered = snapshot.clone();
    {
        let mut entry = tampered.diagnostic_graph.get_entry_mut(node).unwrap();
        entry.set_retained_diagnostic_artifact(None);
        entry.set_causality(None);
    }

    graph.restore_snapshot(&tampered).unwrap();

    assert_eq!(
        graph
            .get_entry(node)
            .unwrap()
            .get_aspect_version()
            .get(ASPECT_A),
        1,
        "restore must still follow checkpoint authority for operational state"
    );
    assert!(
        graph
            .get_entry(node)
            .unwrap()
            .retained_diagnostic_artifact()
            .is_none(),
        "restored authority lane must not rehydrate node-local retained artifacts from the checkpoint image"
    );
    assert!(
        graph.get_entry(node).unwrap().get_causality().is_none(),
        "restored authority lane must not rehydrate causality from the checkpoint image"
    );
}

#[test]
fn restore_snapshot_with_active_policy_prunes_cold_richness_without_changing_operational_truth() {
    // This standalone caller declares the operational serial memory policy.
    let serial_request = worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::new(
            crate::runtime_policy::SignalRuntimePolicy::operational().serial_memory_bytes,
        ),
        worth_execution::CancellationToken::new(),
        None,
    );
    let request_execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let mut runtime = SignalRuntime::builder(SignalGraph::new())
        .with_kernel_defaults()
        .build();
    let node = runtime.graph_mut().node().output_identity().build();
    let mut runtime_ctx = ();

    runtime.set_runtime_policy(SignalRuntimePolicy::development());
    runtime
        .transaction(request_execution, &mut runtime_ctx, |tx| {
            tx.read(node, &|view| {
                Ok(view.finish(
                    NodeEvaluationResult::from_version(version_ab(1, 0))
                        .with_output_identity("restore-policy")
                        .with_label("retained"),
                ))
            })?;
            Ok(())
        })
        .unwrap();

    let explanation = runtime
        .observe()
        .materialize()
        .materialize_explanation_artifact(node)
        .unwrap()
        .0
        .expect("development policy should materialize explanation");
    runtime
        .graph_mut()
        .diagnostics_state_mut()
        .record_explanation_fact(ExplanationFact::from_explanation(&explanation));
    runtime
        .graph_mut()
        .diagnostics_state_mut()
        .record_provenance_fact(ProvenanceFact::from_explanation(&explanation));

    let snapshot = runtime
        .capture_snapshot()
        .expect("snapshot capture should succeed without managed queue bindings");
    assert!(
        snapshot.diagnostics.explanation_facts.contains_key(&node),
        "captured snapshot should include retained explanation richness"
    );

    runtime.set_runtime_policy(
        SignalRuntimePolicy::operational()
            .with_explanation_retention(ArtifactRetentionPolicy::Omit)
            .with_provenance_retention(ArtifactRetentionPolicy::Omit)
            .with_observation_activation(
                worth_foundational::ObservationActivationProfile::Continuous,
            ),
    );
    runtime
        .transaction(request_execution, &mut runtime_ctx, |tx| {
            tx.mark_dirty(node, ASPECT_A)?;
            tx.read(node, &|view| {
                Ok(view.finish(
                    NodeEvaluationResult::from_version(version_ab(2, 0))
                        .with_output_identity("restore-policy-updated"),
                ))
            })?;
            Ok(())
        })
        .unwrap();

    let restore_plan = runtime
        .graph()
        .plan_snapshot_restore(
            &snapshot,
            SnapshotRestoreIntent::restore_runtime_truth_with_active_policy(),
        )
        .unwrap();
    runtime
        .restore_snapshot_with_intent(
            &snapshot,
            SnapshotRestoreIntent::restore_runtime_truth_with_active_policy(),
        )
        .unwrap();

    assert_eq!(
        runtime
            .graph()
            .get_entry(node)
            .unwrap()
            .get_aspect_version()
            .get(ASPECT_A),
        1,
        "active-policy restore should still rewind operational state"
    );
    let (artifact, materialization_mode) = runtime
        .observe()
        .materialize()
        .materialize_explanation_artifact(node)
        .unwrap();
    assert!(artifact.is_none());
    assert_eq!(materialization_mode, DiagnosticsAvailability::OmittedByTier);
    assert!(
        runtime
            .observe()
            .metrics()
            .checkpoint
            .snapshot_restore_count
            >= 1,
        "restore intent should be visible in checkpoint telemetry"
    );
    assert!(
        runtime
            .observe()
            .metrics()
            .checkpoint
            .snapshot_restore_apply_active_policy_count
            >= 1,
        "active-policy restore should be counted explicitly for certification"
    );
    assert_eq!(
        runtime
            .observe()
            .metrics()
            .checkpoint
            .snapshot_restore_shared_delta_node_count,
        restore_plan.dependency_snapshot_delta_node_count(),
        "runtime restore counters should report the same shared-node delta breadth as the canonical restore plan"
    );
    assert_eq!(
        runtime
            .observe()
            .metrics()
            .checkpoint
            .snapshot_restore_coarse_reason_count,
        restore_plan.coarse_reasons().len() as u64,
        "runtime restore counters should report the same coarse restore reason count as the canonical restore plan"
    );
}
