use crate::data::error::SignalError;
use crate::data::graph::SignalGraph;
use crate::data::handle::NodeId;
use crate::logic::transaction::runtime::{
    BranchMergeConflictKind, BranchMergeConflictRecord, BranchMergeConflictSummary,
};

use super::super::super::merge::StructuralMergeCandidateRecord;
use super::super::super::merge::StructuralMergeJournalSlice;
use super::artifact_projection::{node_merge_projection, NodeMergeProjection};

pub(super) struct ConflictClassificationInput<'a> {
    pub(super) source_graph: &'a SignalGraph,
    pub(super) target_graph: &'a SignalGraph,
    pub(super) source_nodes: &'a [NodeId],
    pub(super) identity_matches: &'a std::collections::BTreeMap<NodeId, NodeId>,
    pub(super) source_journal: &'a StructuralMergeJournalSlice,
    pub(super) target_overlap_journal: &'a crate::logic::transaction::BranchMutationJournalSlice,
}

pub(super) fn classify_conflicts(
    input: ConflictClassificationInput<'_>,
) -> Result<Vec<BranchMergeConflictRecord>, SignalError> {
    let mut records = Vec::new();
    for source_node in input.source_nodes {
        let Some(target_node) = input.identity_matches.get(source_node).copied() else {
            continue;
        };
        let source_projection = node_merge_projection(input.source_graph, *source_node)?;
        let target_projection = node_merge_projection(input.target_graph, target_node)?;
        let source_record = input
            .source_journal
            .records
            .iter()
            .find(|record| record.node == *source_node)
            .cloned();
        let target_record = input
            .target_overlap_journal
            .records
            .iter()
            .find(|record| record.node == target_node)
            .cloned();
        if let Some(record) = classify_conflict_record(
            *source_node,
            target_node,
            source_projection,
            target_projection,
            source_record,
            target_record,
        ) {
            records.push(record);
        }
    }
    Ok(records)
}

fn classify_conflict_record(
    source_node: NodeId,
    target_node: NodeId,
    source_projection: Option<NodeMergeProjection>,
    target_projection: Option<NodeMergeProjection>,
    source_structural_record: Option<StructuralMergeCandidateRecord>,
    target_structural_record: Option<StructuralMergeCandidateRecord>,
) -> Option<BranchMergeConflictRecord> {
    let conflict_kinds = classify_conflict_kinds(
        source_projection
            .as_ref()
            .map(|projection| &projection.comparable),
        target_projection
            .as_ref()
            .map(|projection| &projection.comparable),
        source_structural_record.as_ref(),
        target_structural_record.as_ref(),
    );
    (!conflict_kinds.is_empty()).then_some(BranchMergeConflictRecord {
        source_node,
        target_node,
        conflict_kinds,
        source_comparable: source_projection.map(|projection| projection.comparable),
        target_comparable: target_projection.map(|projection| projection.comparable),
        source_structural_record,
        target_structural_record,
    })
}

fn classify_conflict_kinds(
    source_cmp: Option<&crate::logic::transaction::ArtifactMergeComparable>,
    target_cmp: Option<&crate::logic::transaction::ArtifactMergeComparable>,
    source_structural_record: Option<&StructuralMergeCandidateRecord>,
    target_structural_record: Option<&StructuralMergeCandidateRecord>,
) -> Vec<BranchMergeConflictKind> {
    let mut kinds = Vec::new();
    if source_cmp != target_cmp {
        kinds.push(BranchMergeConflictKind::ComparableMismatch);
    }
    if source_cmp.map(|cmp| &cmp.authority) != target_cmp.map(|cmp| &cmp.authority) {
        kinds.push(BranchMergeConflictKind::MergeAuthorityMismatch);
    }
    for (facet, kind) in [
        (
            StructuralConflictFacet::DependencyTopology,
            BranchMergeConflictKind::DependencyTopologyMismatch,
        ),
        (
            StructuralConflictFacet::DependencySnapshot,
            BranchMergeConflictKind::DependencySnapshotMismatch,
        ),
        (
            StructuralConflictFacet::RuntimeArtifact,
            BranchMergeConflictKind::RuntimeArtifactMismatch,
        ),
    ] {
        if structural_delta_conflicts(source_structural_record, target_structural_record, facet) {
            kinds.push(kind);
        }
    }
    kinds
}

#[derive(Clone, Copy)]
enum StructuralConflictFacet {
    DependencyTopology,
    DependencySnapshot,
    RuntimeArtifact,
}

fn structural_delta_conflicts(
    source_record: Option<&StructuralMergeCandidateRecord>,
    target_record: Option<&StructuralMergeCandidateRecord>,
    facet: StructuralConflictFacet,
) -> bool {
    structural_deltas_for_facet(source_record, facet)
        != structural_deltas_for_facet(target_record, facet)
}

fn structural_deltas_for_facet(
    record: Option<&StructuralMergeCandidateRecord>,
    facet: StructuralConflictFacet,
) -> Vec<crate::data::graph::BranchStructuralDelta> {
    record
        .map(|record| {
            record
                .structural_deltas
                .iter()
                .filter(|delta| {
                    matches!(
                        (facet, delta),
                        (
                            StructuralConflictFacet::DependencyTopology,
                            crate::data::graph::BranchStructuralDelta::DependencyTopologyChanged(_)
                        ) | (
                            StructuralConflictFacet::DependencySnapshot,
                            crate::data::graph::BranchStructuralDelta::DependencySnapshotChanged(_)
                        ) | (
                            StructuralConflictFacet::RuntimeArtifact,
                            crate::data::graph::BranchStructuralDelta::RuntimeArtifactChanged(_)
                        )
                    )
                })
                .cloned()
                .collect()
        })
        .unwrap_or_default()
}

pub(super) fn summarize_conflicts(
    records: &[BranchMergeConflictRecord],
) -> BranchMergeConflictSummary {
    let mut summary = BranchMergeConflictSummary {
        total_conflict_count: records.len() as u64,
        ..Default::default()
    };
    for record in records {
        summary.comparable_mismatch_count += u64::from(
            record
                .conflict_kinds
                .contains(&BranchMergeConflictKind::ComparableMismatch),
        );
        summary.dependency_topology_mismatch_count += u64::from(
            record
                .conflict_kinds
                .contains(&BranchMergeConflictKind::DependencyTopologyMismatch),
        );
        summary.dependency_snapshot_mismatch_count += u64::from(
            record
                .conflict_kinds
                .contains(&BranchMergeConflictKind::DependencySnapshotMismatch),
        );
        summary.runtime_artifact_mismatch_count += u64::from(
            record
                .conflict_kinds
                .contains(&BranchMergeConflictKind::RuntimeArtifactMismatch),
        );
        summary.merge_authority_mismatch_count += u64::from(
            record
                .conflict_kinds
                .contains(&BranchMergeConflictKind::MergeAuthorityMismatch),
        );
    }
    let counts = [
        (
            BranchMergeConflictKind::ComparableMismatch,
            summary.comparable_mismatch_count,
        ),
        (
            BranchMergeConflictKind::DependencyTopologyMismatch,
            summary.dependency_topology_mismatch_count,
        ),
        (
            BranchMergeConflictKind::DependencySnapshotMismatch,
            summary.dependency_snapshot_mismatch_count,
        ),
        (
            BranchMergeConflictKind::RuntimeArtifactMismatch,
            summary.runtime_artifact_mismatch_count,
        ),
        (
            BranchMergeConflictKind::MergeAuthorityMismatch,
            summary.merge_authority_mismatch_count,
        ),
    ];
    summary.primary_conflict_kind = counts
        .into_iter()
        .max_by_key(|(_, count)| *count)
        .and_then(|(kind, count)| (count > 0).then_some(kind));
    summary.required_resolution = [
        (
            summary.comparable_mismatch_count,
            crate::logic::transaction::runtime::BranchMergeResolutionRequirement::ReconcileComparableState,
        ),
        (
            summary.dependency_topology_mismatch_count,
            crate::logic::transaction::runtime::BranchMergeResolutionRequirement::ReconcileDependencyTopology,
        ),
        (
            summary.dependency_snapshot_mismatch_count,
            crate::logic::transaction::runtime::BranchMergeResolutionRequirement::ReconcileDependencySnapshot,
        ),
        (
            summary.runtime_artifact_mismatch_count,
            crate::logic::transaction::runtime::BranchMergeResolutionRequirement::ReconcileRuntimeArtifactState,
        ),
        (
            summary.merge_authority_mismatch_count,
            crate::logic::transaction::runtime::BranchMergeResolutionRequirement::ReconcileMergeAuthority,
        ),
    ]
    .into_iter()
    .filter_map(|(count, requirement)| (count > 0).then_some(requirement))
    .collect();
    summary
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::graph::signal_graph::{
        DependencySnapshotStructuralDelta, RuntimeArtifactStructuralDelta,
    };
    use crate::data::graph::BranchStructuralDelta;
    use crate::diagnostics::lineage::LineageArtifactId;

    fn artifact_change(previous: u64, next: u64) -> BranchStructuralDelta {
        BranchStructuralDelta::RuntimeArtifactChanged(RuntimeArtifactStructuralDelta {
            previous_artifact_id: Some(LineageArtifactId(previous)),
            next_artifact_id: Some(LineageArtifactId(next)),
            previous_output_hash: Some(previous.into()),
            next_output_hash: Some(next.into()),
            previous_reuse_basis: None,
            next_reuse_basis: None,
        })
    }

    fn record(node: NodeId, delta: BranchStructuralDelta) -> StructuralMergeCandidateRecord {
        StructuralMergeCandidateRecord {
            node,
            introduced: false,
            state_changed: true,
            dependencies_changed: false,
            dependency_snapshot_changed: false,
            runtime_artifact_changed: true,
            retained_artifact_changed: false,
            causality_changed: false,
            structural_deltas: vec![delta],
        }
    }

    #[test]
    fn branches_reaching_the_same_net_artifact_do_not_conflict() {
        let node = SignalGraph::new().create_node();
        let source = record(node, artifact_change(0, 9));
        let target = record(node, artifact_change(0, 9));

        assert!(classify_conflict_kinds(None, None, Some(&source), Some(&target)).is_empty());
    }

    fn snapshot_change(changed_entry_count: u32) -> BranchStructuralDelta {
        BranchStructuralDelta::DependencySnapshotChanged(DependencySnapshotStructuralDelta {
            previous_entry_count: 3,
            next_entry_count: 4,
            changed_entry_count,
        })
    }

    #[test]
    fn branches_reaching_one_snapshot_along_different_paths_conflict() {
        let node = SignalGraph::new().create_node();
        let source = record(node, snapshot_change(1));
        let target = record(node, snapshot_change(5));

        assert_eq!(
            classify_conflict_kinds(None, None, Some(&source), Some(&target)),
            vec![BranchMergeConflictKind::DependencySnapshotMismatch]
        );
    }

    #[test]
    fn branches_reaching_different_artifacts_conflict() {
        let node = SignalGraph::new().create_node();
        let source = record(node, artifact_change(0, 9));
        let target = record(node, artifact_change(0, 8));

        assert_eq!(
            classify_conflict_kinds(None, None, Some(&source), Some(&target)),
            vec![BranchMergeConflictKind::RuntimeArtifactMismatch]
        );
    }
}
