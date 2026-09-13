use super::SignalGraph;
use crate::data::handle::NodeId;
use crate::data::trace::{CausalityMetadata, ExecutionTraceStamp};

pub(super) fn set_execution_record(graph: &mut SignalGraph, node: NodeId, record: Option<u64>) {
    let mut entry = graph.get_entry_mut(node).unwrap();
    entry.set_execution_trace_stamp(Some(ExecutionTraceStamp {
        execution_record_id: record,
        semantic_segment_id: record,
    }));
}

pub(super) fn set_causality(graph: &mut SignalGraph, node: NodeId, kind: &str) {
    graph
        .set_causality(
            node,
            Some(CausalityMetadata {
                kind: kind.to_owned(),
                fields: Default::default(),
            }),
        )
        .unwrap();
}

fn assert_execution_stamp(graph: &SignalGraph, node: NodeId, record: Option<u64>) {
    let stamp = graph
        .node_execution_trace_stamp(node)
        .unwrap()
        .expect("fixture retains an execution stamp");
    assert_eq!(stamp.execution_record_id, record);
    assert_eq!(stamp.semantic_segment_id, record);
}

fn assert_causality_kind(graph: &SignalGraph, node: NodeId, expected: &str) {
    assert_eq!(
        graph
            .causality_of(node)
            .unwrap()
            .map(|cause| cause.kind.as_str()),
        Some(expected)
    );
}

pub(super) fn assert_populated_cold_facts(
    graph: &SignalGraph,
    aligned: NodeId,
    without_record: NodeId,
    recycled: NodeId,
) {
    assert_execution_stamp(graph, aligned, Some(30));
    assert_execution_stamp(graph, without_record, None);
    assert_causality_kind(graph, aligned, "serde-aligned");
    assert_causality_kind(graph, recycled, "serde-recycled");
}

pub(super) fn assert_forked_cold_facts(
    graph: &SignalGraph,
    inherited: NodeId,
    inherited_without_record: NodeId,
    recycled: NodeId,
) {
    assert_execution_stamp(graph, inherited, Some(63));
    assert_execution_stamp(graph, inherited_without_record, None);
    assert_execution_stamp(graph, recycled, Some(10));
    assert_causality_kind(graph, inherited, "inherited");
    assert_causality_kind(graph, recycled, "child-recycled");
}
