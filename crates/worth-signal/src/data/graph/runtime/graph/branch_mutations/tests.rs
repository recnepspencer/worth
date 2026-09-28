use super::*;

fn artifact_change(previous: u64, next: u64) -> RuntimeArtifactStructuralDelta {
    RuntimeArtifactStructuralDelta {
        previous_artifact_id: Some(LineageArtifactId(previous)),
        next_artifact_id: Some(LineageArtifactId(next)),
        previous_output_hash: Some(previous.into()),
        next_output_hash: Some(next.into()),
        previous_reuse_basis: None,
        next_reuse_basis: None,
    }
}

fn evaluated_times(times: u64) -> BranchMutationRecord {
    let mut record = BranchMutationRecord::default();
    record.mark_introduced();
    for evaluation in 0..times {
        record.mark_state_changed();
        record.mark_retained_artifact_changed();
        record.mark_runtime_artifact_changed(artifact_change(evaluation, evaluation + 1));
        record.mark_causality_changed();
    }
    record
}

#[test]
fn a_node_evaluated_many_times_journals_each_facet_once() {
    let record = evaluated_times(1_000);

    assert_eq!(
        record.structural_deltas,
        vec![
            BranchStructuralDelta::NodeIntroduced,
            BranchStructuralDelta::NodeStateChanged,
            BranchStructuralDelta::RetainedArtifactChanged,
            BranchStructuralDelta::RuntimeArtifactChanged(artifact_change(0, 1_000)),
            BranchStructuralDelta::CausalityChanged,
        ]
    );
    assert!(record.introduced && record.state_changed && record.runtime_artifact_changed);
}

#[test]
fn an_absorbed_record_folds_into_the_net_change_since_the_baseline() {
    let mut earlier = evaluated_times(3);
    let mut later = BranchMutationRecord::default();
    later.mark_state_changed();
    later.mark_runtime_artifact_changed(artifact_change(3, 9));
    later.mark_dependencies_changed(DependencyTopologyDelta::default());

    earlier.absorb(later);

    assert_eq!(earlier, {
        let mut expected = evaluated_times(3);
        expected.mark_runtime_artifact_changed(artifact_change(3, 9));
        expected.mark_dependencies_changed(DependencyTopologyDelta::default());
        expected
    });
    assert!(earlier
        .structural_deltas
        .contains(&BranchStructuralDelta::RuntimeArtifactChanged(
            artifact_change(0, 9)
        )));
}

fn snapshot_change(previous: u32, next: u32, changed: u32) -> DependencySnapshotStructuralDelta {
    DependencySnapshotStructuralDelta {
        previous_entry_count: previous,
        next_entry_count: next,
        changed_entry_count: changed,
    }
}

#[test]
fn repeated_snapshot_changes_journal_the_baseline_latest_and_total_changed() {
    let mut record = BranchMutationRecord::default();
    for evaluation in 0..1_000 {
        record.mark_dependency_snapshot_changed(snapshot_change(
            evaluation,
            evaluation + 1,
            u32::MAX / 600,
        ));
    }

    assert_eq!(
        record.structural_deltas,
        vec![BranchStructuralDelta::DependencySnapshotChanged(
            snapshot_change(0, 1_000, u32::MAX)
        )]
    );
    assert!(record.dependency_snapshot_changed);
}
