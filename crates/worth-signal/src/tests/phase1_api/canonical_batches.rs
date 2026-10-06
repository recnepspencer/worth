use crate::facade::*;
use crate::tests::support::*;

#[test]
fn proof_bearing_batches_and_summaries_canonicalize_their_inputs() {
    let node_a = NodeId::new(7, 1);
    let node_b = NodeId::new(3, 2);
    let changed_regions = CanonicalChangedRegions::new(vec![
        ChangedRegion::new("wing").with_detail("spar"),
        ChangedRegion::new("wing").with_detail("spar"),
        ChangedRegion::new("fuselage"),
    ]);
    let touched_nodes = DedupedNodeBatch::new([node_a, node_b, node_a]);
    let touched_sources = SortedSourceBatch::new([node_a, node_b, node_b]);
    let dirty_delta = DirtyDelta::new(AspectMask::from([ASPECT_A]), changed_regions, touched_nodes);
    let structural_delta = StructuralDelta::new(Some(dirty_delta.clone()), None);
    let patch_plan = PatchPlan::new(vec![node_a, node_b, node_a], structural_delta.clone());
    let touched_scope_summary = TouchedScopeSummary::new(
        vec![
            PartitionSubscription::partition_and_detail("wing", "spar"),
            PartitionSubscription::whole_partition("fuselage"),
            PartitionSubscription::partition_and_detail("wing", "spar"),
        ],
        vec![node_a, node_b, node_a],
        vec![node_a, node_b, node_b],
    );
    let snapshot_batch = PendingSnapshotBatch::from_pairs(vec![
        (node_a, crate::data::dependency::DependencySnapshot::empty()),
        (node_b, crate::data::dependency::DependencySnapshot::empty()),
        (node_a, crate::data::dependency::DependencySnapshot::empty()),
    ]);
    let subscriber_repairs = SubscriberRepairBatch::new(vec![
        SubscriberRepair {
            source: node_a,
            subscribers: DedupedNodeBatch::new([node_b, node_b]),
        },
        SubscriberRepair {
            source: node_b,
            subscribers: DedupedNodeBatch::new([node_a, node_a]),
        },
        SubscriberRepair {
            source: node_a,
            subscribers: DedupedNodeBatch::new([node_a, node_b]),
        },
    ]);
    let desired = DesiredState::new(AspectMask::from([ASPECT_A, ASPECT_B]));
    let dependency_batch = DependencyBatchEdit::from_pairs(vec![
        (
            node_a,
            CanonicalDependencies::new([DependencyEdge::new(node_b, ASPECT_A)]),
        ),
        (
            node_b,
            CanonicalDependencies::new([DependencyEdge::new(node_a, ASPECT_B)]),
        ),
    ]);
    let dirty_batch = DirtyBatch::new(vec![
        DirtyBatchEntry::new(node_a, ASPECT_A, vec![ChangedRegion::new("wing")]),
        DirtyBatchEntry::new(
            node_a,
            ASPECT_A,
            vec![ChangedRegion::new("wing"), ChangedRegion::new("fuselage")],
        ),
        DirtyBatchEntry::without_regions(node_b, ASPECT_B),
    ]);
    let semantic_batch_commit = ChangeBatchAdmission::new(dirty_batch.clone());
    let locality = LocalityFootprint::new(
        vec![
            PartitionSubscription::partition_and_detail("wing", "spar"),
            PartitionSubscription::whole_partition("fuselage"),
            PartitionSubscription::partition_and_detail("wing", "spar"),
        ],
        vec![node_a, node_b, node_a],
        vec![node_a, node_b, node_b],
    );
    let snapshot_commit = SnapshotBatchCommit::from_pairs(vec![
        (node_a, crate::data::dependency::DependencySnapshot::empty()),
        (node_b, crate::data::dependency::DependencySnapshot::empty()),
        (node_a, crate::data::dependency::DependencySnapshot::empty()),
    ]);

    assert_eq!(dirty_delta.changed_regions.as_slice().len(), 2);
    assert_eq!(dirty_delta.touched_nodes.as_slice(), &[node_b, node_a]);
    assert!(!structural_delta.is_empty());
    assert!(!patch_plan.is_empty());
    assert_eq!(patch_plan.target_nodes.as_slice(), &[node_b, node_a]);
    assert_eq!(touched_sources.as_slice(), &[node_b, node_a]);
    assert_eq!(touched_scope_summary.seed_scopes.len(), 2);
    assert_eq!(
        touched_scope_summary.touched_nodes.as_slice(),
        &[node_b, node_a]
    );
    assert_eq!(
        touched_scope_summary.touched_sources.as_slice(),
        &[node_b, node_a]
    );
    assert_eq!(snapshot_batch.as_slice().len(), 2);
    assert_eq!(dependency_batch.as_slice().len(), 2);
    assert_eq!(dirty_batch.as_slice().len(), 2);
    assert_eq!(dirty_batch.changed_regions().as_slice().len(), 2);
    assert_eq!(dirty_batch.locality_footprint().partitions.len(), 2);
    assert_eq!(dirty_batch.touched_sources().as_slice(), &[node_b, node_a]);
    assert_eq!(locality.partitions.len(), 2);
    assert_eq!(locality.nodes.as_slice(), &[node_b, node_a]);
    assert_eq!(
        semantic_batch_commit.changed_aspects.bits(),
        AspectMask::from([ASPECT_A, ASPECT_B]).bits()
    );
    assert_eq!(semantic_batch_commit.locality.partitions.len(), 2);
    assert_eq!(snapshot_commit.target_nodes().as_slice(), &[node_b, node_a]);
    assert_eq!(subscriber_repairs.as_slice().len(), 2);
    assert_eq!(
        desired.value().bits(),
        AspectMask::from([ASPECT_A, ASPECT_B]).bits()
    );
}
