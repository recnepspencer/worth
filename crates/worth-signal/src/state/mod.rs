mod authority;
mod diagnostics;
mod lifecycle;
mod restore;
mod snapshot;

pub use self::authority::{
    SignalCheckpointArena, SignalCheckpointAuthority, SignalCheckpointImage, SignalCheckpointSlot,
    SignalCheckpointTopology,
};
pub use self::diagnostics::SignalSnapshotDiagnostics;
pub use self::lifecycle::SnapshotRestoreCoarseReason;
pub use self::lifecycle::{
    SignalBranchHandle, SignalBranchId, SignalSnapshotId, SignalSnapshotMeta,
    SnapshotArtifactRestoreMode, SnapshotArtifactRetentionPolicy, SnapshotDependencyRestoreMode,
    SnapshotRestoreIntent, SnapshotStateRestoreMode,
};
pub use self::restore::{
    CheckpointRestoreSnapshotBatch, RestoreDeltaAccounting, SnapshotRestorePlan,
};
pub use self::snapshot::SignalSnapshotV1;

#[cfg(test)]
mod tests {
    use super::{SignalCheckpointAuthority, SignalCheckpointSlot};
    use crate::data::graph::{DependencyEdgeStore, SubscriberEdgeStore};
    use crate::data::node::NodeEntry;
    use crate::diagnostics::state::DiagnosticsState;

    #[derive(serde::Serialize)]
    struct LegacyCheckpointSlot {
        entry: Option<NodeEntry>,
        generation: u32,
        retired: bool,
    }

    #[derive(serde::Serialize)]
    struct LegacyCheckpointArena {
        slots: Vec<LegacyCheckpointSlot>,
        free_list: Vec<u32>,
        active_nodes: u32,
    }

    #[derive(serde::Serialize)]
    struct LegacyCheckpointTopology {
        dependency_edges: DependencyEdgeStore,
        subscriber_edges: SubscriberEdgeStore,
    }

    #[derive(serde::Serialize)]
    struct LegacyCheckpointAuthority {
        arena: LegacyCheckpointArena,
        topology: LegacyCheckpointTopology,
        diagnostics: DiagnosticsState,
    }

    #[test]
    fn checkpoint_slot_deserializes_legacy_entry_payload() {
        let mut entry = NodeEntry::new();
        entry.set_runtime_artifact_state(Some(crate::data::trace::RuntimeArtifactState::new(
            crate::data::trace::RuntimeArtifactHot {
                output_hash: crate::data::core_profile::StableHashValue::MAX,
                ..Default::default()
            },
            Default::default(),
        )));
        let expected = entry.to_checkpoint_image();
        let legacy = LegacyCheckpointSlot {
            entry: Some(entry),
            generation: 7,
            retired: false,
        };

        let encoded = serde_json::to_vec(&legacy).expect("serialize legacy checkpoint slot");
        let decoded: SignalCheckpointSlot =
            serde_json::from_slice(&encoded).expect("deserialize legacy checkpoint slot");

        assert!(
            decoded.node.is_some(),
            "legacy entry payload should be bridged"
        );
        assert_eq!(decoded.generation, 7);
        assert!(!decoded.retired);
        assert_eq!(decoded.node, Some(expected));
    }

    #[test]
    fn checkpoint_authority_deserializes_legacy_entry_payloads() {
        let legacy = LegacyCheckpointAuthority {
            arena: LegacyCheckpointArena {
                slots: vec![LegacyCheckpointSlot {
                    entry: Some(NodeEntry::new()),
                    generation: 3,
                    retired: false,
                }],
                free_list: Vec::new(),
                active_nodes: 1,
            },
            topology: LegacyCheckpointTopology {
                dependency_edges: DependencyEdgeStore::default(),
                subscriber_edges: SubscriberEdgeStore::default(),
            },
            diagnostics: DiagnosticsState::default(),
        };

        let encoded = serde_json::to_vec(&legacy).expect("serialize legacy checkpoint authority");
        let decoded: SignalCheckpointAuthority =
            serde_json::from_slice(&encoded).expect("deserialize legacy checkpoint authority");

        assert_eq!(decoded.arena.active_nodes, 1);
        assert_eq!(decoded.arena.slots.len(), 1);
        assert!(decoded.arena.slots[0].node.is_some());
    }

    #[test]
    fn checkpoint_slot_serializes_new_node_image_boundary() {
        let slot = SignalCheckpointSlot {
            node: Some(NodeEntry::new().to_checkpoint_image()),
            generation: 11,
            retired: false,
        };

        let encoded = serde_json::to_value(&slot).expect("serialize checkpoint slot");

        assert!(
            encoded.get("node").is_some(),
            "current checkpoint schema must serialize the explicit node image boundary"
        );
        assert!(
            encoded.get("entry").is_none(),
            "current checkpoint schema must not emit the legacy in-memory entry field"
        );
    }

    #[test]
    fn checkpoint_slot_rejects_duplicate_current_and_legacy_carriers() {
        let payload = r#"{"node":null,"entry":null,"generation":1,"retired":false}"#;
        assert!(serde_json::from_str::<SignalCheckpointSlot>(payload).is_err());
        let payload = r#"{"node":null,"node":null,"generation":1,"retired":false}"#;
        assert!(serde_json::from_str::<SignalCheckpointSlot>(payload).is_err());
    }
}
