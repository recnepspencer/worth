use crate::facade::*;
use crate::tests::support::*;

#[test]
fn dependency_snapshot_clone_shares_backing_storage() {
    let mut snapshot = crate::data::dependency::DependencySnapshot::empty();
    snapshot.record(NodeId::new(1, 0), ASPECT_A, 7, None);
    snapshot.record(NodeId::new(2, 0), ASPECT_B, 11, None);

    let cloned = snapshot.clone();

    assert!(std::sync::Arc::ptr_eq(
        &snapshot.shared_entries(),
        &cloned.shared_entries()
    ));
    assert_eq!(snapshot.entries(), cloned.entries());
}

#[test]
fn replacing_dependency_snapshot_reports_delta() {
    let mut graph = SignalGraph::new();
    let node = graph.node().build();
    let source = graph.node().build();

    let mut baseline = crate::data::dependency::DependencySnapshot::empty();
    baseline.record(source, ASPECT_A, 3, None);
    graph.set_dep_snapshot(node, baseline).unwrap();

    let mut updated = crate::data::dependency::DependencySnapshot::empty();
    updated.record(source, ASPECT_A, 5, None);
    updated.record(source, ASPECT_B, 7, None);
    let delta = graph
        .replace_dep_snapshot_committed(
            node,
            crate::data::dependency::CommittedSnapshotUpdate::Replace(
                crate::data::dependency::ReplacementSnapshotUpdate::from_snapshot(updated),
            ),
        )
        .unwrap();

    assert_eq!(delta.node, node);
    assert_eq!(delta.previous_entry_count, 1);
    assert_eq!(delta.next_entry_count, 2);
    assert_eq!(delta.changed_entry_count, 2);
    assert!(delta.changed());
}

#[test]
fn replacing_identical_dependency_snapshot_is_a_noop() {
    let mut graph = SignalGraph::new();
    let node = graph.node().build();
    let source = graph.node().build();

    let mut baseline = crate::data::dependency::DependencySnapshot::empty();
    baseline.record(source, ASPECT_A, 3, None);
    graph.set_dep_snapshot(node, baseline.clone()).unwrap();
    let first_id = graph.get_entry(node).unwrap().get_dep_snapshot_id();
    let delta = graph
        .replace_dep_snapshot_committed(
            node,
            crate::data::dependency::CommittedSnapshotUpdate::Replace(
                crate::data::dependency::ReplacementSnapshotUpdate::from_snapshot(baseline),
            ),
        )
        .unwrap();
    let second_id = graph.get_entry(node).unwrap().get_dep_snapshot_id();

    assert_eq!(first_id, second_id);
    assert_eq!(delta.changed_entry_count, 0);
    assert!(!delta.changed());
}

#[test]
fn dependency_snapshot_version_only_update_preserves_shape() {
    let source_a = NodeId::new(1, 0);
    let source_b = NodeId::new(2, 0);
    let mut baseline = crate::data::dependency::DependencySnapshot::empty();
    baseline.record(source_a, ASPECT_A, 3, None);
    baseline.record(source_b, ASPECT_B, 7, None);

    let updated = baseline.with_updated_versions(&[5, 7]);
    let delta = crate::data::dependency::SnapshotDeltaRecord::between(
        NodeId::new(9, 0),
        &baseline,
        &crate::data::dependency::SharedDependencySnapshot::new(updated.clone()),
    );

    assert_eq!(baseline.entries().len(), updated.entries().len());
    assert_eq!(
        baseline
            .entries()
            .iter()
            .map(|entry| entry.sort_key())
            .collect::<Vec<_>>(),
        updated
            .entries()
            .iter()
            .map(|entry| entry.sort_key())
            .collect::<Vec<_>>()
    );
    assert_eq!(updated.entries()[0].cached_version, 5);
    assert_eq!(updated.entries()[1].cached_version, 7);
    assert_eq!(delta.changed_entry_count, 1);
    assert!(delta.changed());
}

#[test]
fn shared_dependency_snapshot_reports_storage_sharing_without_implying_semantics() {
    let source = NodeId::new(1, 0);
    let mut baseline = crate::data::dependency::DependencySnapshot::empty();
    baseline.record(source, ASPECT_A, 3, None);

    let shared_left = crate::data::dependency::SharedDependencySnapshot::new(baseline.clone());
    let shared_right = crate::data::dependency::SharedDependencySnapshot::new(baseline.clone());

    assert!(
        baseline.shares_storage_with(shared_left.snapshot()),
        "shared snapshot wrapping should preserve shared backing"
    );
    assert!(
        shared_left.shares_storage_with(&shared_right),
        "cloned snapshots should report shared backing explicitly"
    );

    let mut shape_store = crate::data::dependency::DependencySnapshotShapeStore::default();
    let replace = crate::data::dependency::CommittedSnapshotUpdate::Replace(
        crate::data::dependency::ReplacementSnapshotUpdate::from_snapshot(
            shared_left.into_snapshot(),
        ),
    );
    let basis = crate::data::dependency::StableShapeSnapshotBasis::prove(
        &crate::data::dependency::DependencyInputScan::stable_shape(
            NodeId::new(0, 0),
            crate::data::dependency::DependencySnapshotId::EMPTY,
            1,
            1,
            vec![5],
        ),
        baseline.shape().intern(&mut shape_store),
    )
    .expect("stable shape proof should exist");
    let version_only = crate::data::dependency::CommittedSnapshotUpdate::VersionOnly(
        crate::data::dependency::VersionOnlySnapshotUpdate::from_basis_and_versions(
            basis.clone(),
            crate::data::dependency::VersionVector::from_scan(
                &basis,
                &crate::data::dependency::DependencyInputScan::stable_shape(
                    NodeId::new(0, 0),
                    crate::data::dependency::DependencySnapshotId::EMPTY,
                    1,
                    1,
                    vec![5],
                ),
            ),
        ),
    );

    assert_eq!(
        replace.storage_strategy(),
        crate::data::dependency::SnapshotStorageStrategy::SharedReplacement
    );
    assert_eq!(
        version_only.storage_strategy(),
        crate::data::dependency::SnapshotStorageStrategy::VersionOnlyDelta
    );
}

#[test]
fn snapshot_storage_telemetry_distinguishes_replacement_from_version_only_delta() {
    let mut graph = SignalGraph::new();
    let node = graph.node().build();
    let source = graph.node().build();

    let mut baseline = crate::data::dependency::DependencySnapshot::empty();
    baseline.record(source, ASPECT_A, 3, None);
    graph.set_dep_snapshot(node, baseline.clone()).unwrap();

    let mut replaced = crate::data::dependency::DependencySnapshot::empty();
    replaced.record(source, ASPECT_A, 5, None);
    replaced.record(source, ASPECT_B, 7, None);
    graph
        .replace_dep_snapshot_committed(
            node,
            crate::data::dependency::CommittedSnapshotUpdate::Replace(
                crate::data::dependency::ReplacementSnapshotUpdate::from_snapshot(replaced),
            ),
        )
        .unwrap();

    let mut proof_shape_store = crate::data::dependency::DependencySnapshotShapeStore::default();
    let current_snapshot = graph.get_dep_snapshot(node).unwrap().clone();
    let basis = crate::data::dependency::StableShapeSnapshotBasis::prove(
        &crate::data::dependency::DependencyInputScan::stable_shape(
            node,
            graph.get_entry(node).unwrap().get_dep_snapshot_id(),
            current_snapshot.entries().len(),
            current_snapshot.entries().len(),
            vec![11, 13],
        ),
        current_snapshot.shape().intern(&mut proof_shape_store),
    )
    .expect("stable shape proof should exist for version-only update");
    graph
        .replace_dep_snapshot_committed(
            node,
            crate::data::dependency::CommittedSnapshotUpdate::VersionOnly(
                crate::data::dependency::VersionOnlySnapshotUpdate::from_basis_and_versions(
                    basis.clone(),
                    crate::data::dependency::VersionVector::from_scan(
                        &basis,
                        &crate::data::dependency::DependencyInputScan::stable_shape(
                            node,
                            graph.get_entry(node).unwrap().get_dep_snapshot_id(),
                            current_snapshot.entries().len(),
                            current_snapshot.entries().len(),
                            vec![11, 13],
                        ),
                    ),
                ),
            ),
        )
        .unwrap();

    let storage = graph.observe().metrics().storage;
    assert!(
        storage.shared_snapshot_replacement_count >= 2,
        "snapshot telemetry should count full shared replacement boundaries"
    );
    assert!(
        storage.version_only_snapshot_update_count >= 1,
        "snapshot telemetry should count version-only delta boundaries separately"
    );
}

#[test]
fn version_only_commit_preserves_stable_shape_change_kind() {
    let mut graph = SignalGraph::new();
    let node = graph.node().build();
    let source = graph.node().build();

    let mut baseline = crate::data::dependency::DependencySnapshot::empty();
    baseline.record(source, ASPECT_A, 3, None);
    baseline.record(source, ASPECT_B, 7, None);
    graph.set_dep_snapshot(node, baseline.clone()).unwrap();

    let mut proof_shape_store = crate::data::dependency::DependencySnapshotShapeStore::default();
    let current_snapshot = graph.get_dep_snapshot(node).unwrap().clone();
    let next_versions = vec![11, 13];
    let basis = crate::data::dependency::StableShapeSnapshotBasis::prove(
        &crate::data::dependency::DependencyInputScan::stable_shape(
            node,
            graph.get_entry(node).unwrap().get_dep_snapshot_id(),
            current_snapshot.entries().len(),
            current_snapshot.entries().len(),
            next_versions.clone(),
        ),
        current_snapshot.shape().intern(&mut proof_shape_store),
    )
    .expect("stable shape proof should exist for version-only update");

    let delta = graph
        .replace_dep_snapshot_committed(
            node,
            crate::data::dependency::CommittedSnapshotUpdate::VersionOnly(
                crate::data::dependency::VersionOnlySnapshotUpdate::from_basis_and_versions(
                    basis.clone(),
                    crate::data::dependency::VersionVector::from_scan(
                        &basis,
                        &crate::data::dependency::DependencyInputScan::stable_shape(
                            node,
                            graph.get_entry(node).unwrap().get_dep_snapshot_id(),
                            current_snapshot.entries().len(),
                            current_snapshot.entries().len(),
                            next_versions,
                        ),
                    ),
                ),
            ),
        )
        .unwrap();

    assert_eq!(
        delta.change_kind,
        crate::data::dependency::SnapshotChangeKind::StableShapeVersionOnly
    );
}

#[test]
fn set_dep_snapshot_uses_version_only_delta_when_snapshot_shape_is_stable() {
    let mut graph = SignalGraph::new();
    let node = graph.node().build();
    let source_a = graph.node().build();
    let source_b = graph.node().build();

    let mut baseline = crate::data::dependency::DependencySnapshot::empty();
    baseline.record(source_a, ASPECT_A, 3, None);
    baseline.record(source_b, ASPECT_B, 7, None);
    graph.set_dep_snapshot(node, baseline).unwrap();

    let mut version_only = crate::data::dependency::DependencySnapshot::empty();
    version_only.record(source_a, ASPECT_A, 5, None);
    version_only.record(source_b, ASPECT_B, 11, None);
    graph.set_dep_snapshot(node, version_only).unwrap();

    let storage = graph.observe().metrics().storage;
    assert_eq!(
        storage.shared_snapshot_replacement_count, 1,
        "initial snapshot install should be the only full replacement when shape stays stable"
    );
    assert_eq!(
        storage.version_only_snapshot_update_count, 1,
        "stable-shape snapshot rewrite should narrow to a version-only delta"
    );
}

#[test]
fn derive_dependency_snapshot_restore_batch_uses_version_only_delta_for_shared_shape() {
    let mut current = SignalGraph::new();
    let source_a = current.node().build();
    let source_b = current.node().build();
    let target = current.node().build();

    let mut baseline = crate::data::dependency::DependencySnapshot::empty();
    baseline.record(source_a, ASPECT_A, 3, None);
    baseline.record(source_b, ASPECT_B, 7, None);
    current.set_dep_snapshot(target, baseline).unwrap();

    let mut restored = current.clone();
    let mut updated = crate::data::dependency::DependencySnapshot::empty();
    updated.record(source_a, ASPECT_A, 5, None);
    updated.record(source_b, ASPECT_B, 11, None);
    restored.set_dep_snapshot(target, updated).unwrap();

    let batch = current
        .derive_dependency_snapshot_restore_batch(&restored)
        .unwrap();
    let entries = batch.pending().as_slice();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].node, target);
    assert_eq!(
        entries[0].update.storage_strategy(),
        crate::data::dependency::SnapshotStorageStrategy::VersionOnlyDelta
    );
    assert_eq!(entries[0].delta.changed_entry_count, 2);
}
