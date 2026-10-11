#[cfg(target_pointer_width = "64")]
use crate::data::graph::checked_segment_component_for_test;
use crate::facade::*;

#[test]
fn node_id_round_trip_supports_high_u32_generation_values() {
    let node = NodeId::new(7, u32::MAX - 1);
    let encoded = serde_json::to_string(&node).unwrap();
    let decoded: NodeId = serde_json::from_str(&encoded).unwrap();
    assert_eq!(decoded, node);
}

#[test]
fn wrapped_slots_are_retired_and_never_reused() {
    let mut graph = SignalGraph::new();
    let original = graph.create_node();
    graph
        .force_slot_generation_for_test(original.index(), u32::MAX)
        .unwrap();
    let wrapping = NodeId::new(original.index(), u32::MAX);
    graph.unregister_node(wrapping).unwrap();

    assert!(graph.is_slot_retired_for_test(original.index()).unwrap());
    let replacement = graph.create_node();

    assert_ne!(replacement.index(), original.index());
    assert!(!graph.is_alive(original));
    assert!(!graph.is_alive(wrapping));
    assert!(graph.is_alive(replacement));
}

// The refusal cannot happen by type on 32 bits: usize cannot represent
// a value past u32::MAX.
#[cfg(target_pointer_width = "64")]
#[test]
fn edge_store_segment_width_seam_rejects_values_past_u32_capacity() {
    let err = checked_segment_component_for_test(u32::MAX as usize + 1).unwrap_err();
    assert!(format!("{err}").contains("exceeds u32 capacity"));
}
