use super::*;
use crate::diagnostics::lineage::{
    ArtifactTransitionKind, InvalidationCause, LineageArtifactId, SnapshotRestoreKind,
};
use crate::logic::transaction::{ArtifactMergeAction, BranchMergeConflictKind, MergeDecisionBasis};
use crate::state::SignalSnapshotId;

#[test]
fn remaining_lineage_variants_charge_their_actual_retained_payloads() {
    let (_, node) = emitted_lineage();
    for switch in [false, true] {
        let mut first = String::with_capacity(1024);
        first.push_str("first");
        let mut second = String::with_capacity(2048);
        second.push_str("second");
        let expected = first.capacity() + second.capacity();
        let kind = if switch {
            LineageRecordKind::BranchSwitch {
                from_branch_id: SignalBranchId(0),
                to_branch_id: SignalBranchId(1),
                from_branch_display_name: first,
                to_branch_display_name: second,
            }
        } else {
            LineageRecordKind::BranchFork {
                created_branch_id: SignalBranchId(1),
                parent_branch_id: SignalBranchId(0),
                created_branch_display_name: first,
                parent_branch_display_name: second,
            }
        };
        assert_eq!(
            kind.retained_heap_charge(&mut Work::new(100))
                .unwrap()
                .bytes(),
            expected as u64
        );
    }
    let mut resolved_conflict_kinds = Vec::with_capacity(128);
    resolved_conflict_kinds.push(BranchMergeConflictKind::RuntimeArtifactMismatch);
    let expected =
        resolved_conflict_kinds.capacity() * std::mem::size_of::<BranchMergeConflictKind>();
    let kind = LineageRecordKind::ArtifactMerge {
        source_node: node,
        target_node: Some(node),
        source_branch_id: SignalBranchId(1),
        target_branch_id: SignalBranchId(0),
        source_artifact_id: None,
        target_artifact_id_before: None,
        target_artifact_id_after: None,
        merge_action: ArtifactMergeAction::Adopted,
        decision_basis: MergeDecisionBasis::SourceAuthorityAdopted,
        merge_kind: BranchMergeKind::Applied,
        divergence: BranchMergeDivergence::None,
        merge_strategy: BranchMergeStrategy::AdoptSourceHead,
        reconciliation_policy: BranchMergeReconciliationPolicy::built_in_default(),
        resolved_conflict_kinds,
    };
    assert_eq!(
        kind.retained_heap_charge(&mut Work::new(100))
            .unwrap()
            .bytes(),
        expected as u64
    );
    for kind in [
        LineageRecordKind::SnapshotRestore {
            snapshot_id: SignalSnapshotId(1),
            node: Some(node),
            artifact_id: None,
            restore_kind: SnapshotRestoreKind::PerNodeArtifact,
        },
        LineageRecordKind::Invalidation {
            node,
            artifact_id: LineageArtifactId(1),
            cause: InvalidationCause::DirectDependencyChanged {
                dependency: node,
                aspect_index: 0,
            },
        },
    ] {
        assert_eq!(
            kind.retained_heap_charge(&mut Work::new(100)).unwrap(),
            Charge::ZERO
        );
    }
    assert_eq!(
        ArtifactTransitionKind::Replaced
            .retained_heap_charge(&mut Work::new(1))
            .unwrap(),
        Charge::ZERO
    );
}
