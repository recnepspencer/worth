use super::*;
use crate::data::handle::NodeId;
use crate::diagnostics::lineage::{
    ArtifactTransitionKind, InvalidationCause, LineageArtifactId, SnapshotRestoreKind,
};
use crate::state::{SignalBranchId, SignalSnapshotId};

#[test]
fn remaining_lineage_variants_charge_their_actual_retained_payloads() {
    let node = NodeId::new(0, 1);
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
            kind.retained_heap_charge(&mut Work::new(Some(100)))
                .unwrap()
                .bytes(),
            expected as u64
        );
    }
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
            kind.retained_heap_charge(&mut Work::new(Some(100)))
                .unwrap(),
            Charge::ZERO
        );
    }
    assert_eq!(
        ArtifactTransitionKind::Replaced
            .retained_heap_charge(&mut Work::new(Some(1)))
            .unwrap(),
        Charge::ZERO
    );
}
