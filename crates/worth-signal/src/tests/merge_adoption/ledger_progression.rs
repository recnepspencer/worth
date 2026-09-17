use crate::facade::{
    ArtifactMergeAction, CanonicalChangedRegions, NodeEvaluationResult, RetainedDiagnosticArtifact,
    SignalGraph, SignalRuntime,
};
use crate::tests::support::{version_ab, ASPECT_A};

#[test]
fn repeated_merge_advances_source_branch_ledger_boundary() {
    let mut runtime = SignalRuntime::builder(SignalGraph::new())
        .with_kernel_defaults()
        .build();
    let main = runtime.observe().current_branch();
    let feature = runtime.create_branch("feature-repeat-merge").unwrap();
    let mut runtime_ctx = ();

    runtime.switch_branch(feature.clone()).unwrap();
    let first = runtime.graph_mut().node().output_identity().build();
    runtime
        .transaction(&mut runtime_ctx, |tx| {
            tx.read(first, &|view| {
                Ok(view.finish(
                    NodeEvaluationResult::from_version(version_ab(21, 0))
                        .with_output_identity("first-merge"),
                ))
            })?;
            Ok(())
        })
        .unwrap();

    runtime.switch_branch(main.clone()).unwrap();
    let first_merge = runtime
        .merge_branch_raw(feature.clone(), main.clone())
        .unwrap();
    assert_eq!(
        first_merge.records.len(),
        1,
        "first merge should only report the initial source-only node"
    );

    runtime.switch_branch(feature.clone()).unwrap();
    let second = runtime.graph_mut().node().output_identity().build();
    runtime
        .transaction(&mut runtime_ctx, |tx| {
            tx.read(second, &|view| {
                Ok(view.finish(
                    NodeEvaluationResult::from_version(version_ab(22, 0))
                        .with_output_identity("second-merge"),
                ))
            })?;
            Ok(())
        })
        .unwrap();

    runtime.switch_branch(main).unwrap();
    let second_merge = runtime
        .merge_branch_raw(feature, runtime.observe().current_branch())
        .unwrap();

    assert!(
        second_merge.counters.source_slice_breadth
            == second_merge.planned_candidates.nodes.len() as u64,
        "repeated merge should continue using the source ledger candidate set"
    );
    assert!(
        second_merge
            .records
            .iter()
            .all(|record| record.source_node != first),
        "source ledger should advance past already-merged nodes"
    );
    assert!(
        second_merge
            .records
            .iter()
            .any(|record| record.source_node == second),
        "new source mutations should remain merge-visible"
    );
}

#[test]
fn retained_only_branch_churn_does_not_force_merge_replanning() {
    let mut runtime = SignalRuntime::builder(SignalGraph::new())
        .with_kernel_defaults()
        .build();
    let main = runtime.observe().current_branch();
    let feature = runtime.create_branch("feature-retained-only").unwrap();
    let mut runtime_ctx = ();

    runtime.switch_branch(feature.clone()).unwrap();
    let source_only = runtime.graph_mut().node().output_identity().build();
    runtime
        .transaction(&mut runtime_ctx, |tx| {
            tx.read(source_only, &|view| {
                Ok(view.finish(
                    NodeEvaluationResult::from_version(version_ab(31, 0))
                        .with_output_identity("retained-only"),
                ))
            })?;
            Ok(())
        })
        .unwrap();

    runtime.switch_branch(main.clone()).unwrap();
    let _ = runtime
        .merge_branch_raw(feature.clone(), main.clone())
        .unwrap();

    runtime.switch_branch(feature.clone()).unwrap();
    {
        let mut graph = runtime.graph_mut();
        let mut entry = graph.get_entry_mut(source_only).unwrap();
        entry.set_retained_diagnostic_artifact(Some(RetainedDiagnosticArtifact {
            changed_regions: CanonicalChangedRegions::new([]),
            labels: vec!["retained-only-label".to_string()],
            keyed_family: None,
            keyed_key: None,
            reuse_certification: None,
            reuse_boundary_context: None,
        }));
        drop(entry);
        graph.record_branch_mutation_retained_artifact(source_only);
    }

    runtime.switch_branch(main).unwrap();
    let result = runtime
        .merge_branch_raw(feature, runtime.observe().current_branch())
        .unwrap();

    assert!(
        result.planned_candidates.nodes.is_empty(),
        "retained-only churn should produce an explicit empty merge candidate set, not a whole-branch fallback"
    );
    assert!(
        result.records.is_empty(),
        "diagnostics-only retained churn should not create merge reconciliation work"
    );
    assert!(
        result.counters.final_candidate_breadth == 0,
        "diagnostics-only retained churn must report zero final candidate breadth"
    );
}

#[test]
fn merge_branch_equivalent_runtime_state_ignores_retained_artifact_richness() {
    let mut runtime = SignalRuntime::builder(SignalGraph::new())
        .with_kernel_defaults()
        .build();
    let shared = runtime.graph_mut().node().output_identity().build();
    let mut runtime_ctx = ();

    runtime
        .transaction(&mut runtime_ctx, |tx| {
            tx.read(shared, &|view| {
                Ok(view.finish(
                    NodeEvaluationResult::from_version(version_ab(35, 0))
                        .with_output_identity("retained-agnostic"),
                ))
            })?;
            Ok(())
        })
        .unwrap();

    {
        let mut graph = runtime.graph_mut();
        let mut entry = graph.get_entry_mut(shared).unwrap();
        entry.set_retained_diagnostic_artifact(Some(RetainedDiagnosticArtifact {
            changed_regions: CanonicalChangedRegions::new([]),
            labels: vec!["main-label".to_string()],
            keyed_family: None,
            keyed_key: None,
            reuse_certification: None,
            reuse_boundary_context: None,
        }));
    }

    let main = runtime.observe().current_branch();
    let feature = runtime.create_branch("feature-runtime-equivalent").unwrap();
    runtime.switch_branch(feature.clone()).unwrap();

    runtime
        .transaction(&mut runtime_ctx, |tx| {
            tx.mark_dirty(shared, ASPECT_A)?;
            tx.read(shared, &|view| {
                Ok(view.finish(
                    NodeEvaluationResult::from_version(version_ab(35, 0))
                        .with_output_identity("retained-agnostic"),
                ))
            })?;
            Ok(())
        })
        .unwrap();

    {
        let mut graph = runtime.graph_mut();
        let mut entry = graph.get_entry_mut(shared).unwrap();
        entry.set_retained_diagnostic_artifact(Some(RetainedDiagnosticArtifact {
            changed_regions: CanonicalChangedRegions::new([]),
            labels: vec!["feature-label".to_string()],
            keyed_family: Some("family".to_string()),
            keyed_key: Some("key".to_string()),
            reuse_certification: None,
            reuse_boundary_context: None,
        }));
        drop(entry);
        graph.record_branch_mutation_retained_artifact(shared);
    }

    runtime.switch_branch(main).unwrap();
    let result = runtime
        .merge_branch_raw(feature, runtime.observe().current_branch())
        .unwrap();
    let shared_record = result
        .records
        .iter()
        .find(|record| record.source_node == shared)
        .expect("shared node should still be part of the merge summary");

    assert!(
        matches!(
            shared_record.action,
            ArtifactMergeAction::EquivalentUnchanged
        ),
        "merge comparability should be driven by runtime state, not retained richness"
    );
}

/// Capturing a snapshot observes a branch; it must not consume the pending
/// merge ledger. The worker-first root mints snapshots after every mutation,
/// so a capture that emptied the ledger would make every later merge adopt
/// nothing. Both capture paths (active branch, stored branch) are covered.
#[test]
fn snapshot_capture_keeps_pending_source_mutations_merge_visible() {
    let mut runtime = SignalRuntime::builder(SignalGraph::new())
        .with_kernel_defaults()
        .build();
    let main = runtime.observe().current_branch();
    let feature = runtime
        .create_branch("feature-snapshot-observation")
        .unwrap();
    let mut runtime_ctx = ();

    runtime.switch_branch(feature.clone()).unwrap();
    let edited = runtime.graph_mut().node().output_identity().build();
    runtime
        .transaction(&mut runtime_ctx, |tx| {
            tx.read(edited, &|view| {
                Ok(view.finish(
                    NodeEvaluationResult::from_version(version_ab(31, 0))
                        .with_output_identity("observed-before-merge"),
                ))
            })?;
            Ok(())
        })
        .unwrap();

    let active_snapshot = runtime
        .capture_snapshot()
        .expect("active snapshot capture should succeed");
    runtime.switch_branch(main.clone()).unwrap();
    let stored_snapshot = runtime
        .capture_branch_snapshot(feature.clone())
        .expect("stored branch snapshot capture should succeed");
    assert_ne!(
        active_snapshot.meta.snapshot_id, stored_snapshot.meta.snapshot_id,
        "each capture mints its own snapshot"
    );

    let merge = runtime.merge_branch_raw(feature, main).unwrap();
    assert!(
        merge
            .records
            .iter()
            .any(|record| record.source_node == edited
                && matches!(record.action, ArtifactMergeAction::IntroducedIntoTarget)),
        "snapshot captures are observations and must leave the edit merge-visible: {:?}",
        merge
            .records
            .iter()
            .map(|record| (record.source_node, record.action))
            .collect::<Vec<_>>()
    );
}
