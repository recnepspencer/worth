use super::super::SignalGraph;
use crate::data::aspect::{Aspect, AspectVersion};
use crate::data::node::{EvaluationCondition, NodeEvaluationConfig};

#[test]
fn warm_wire_rows_preserve_nondefault_installed_definition() {
    let mut graph = SignalGraph::new();
    let config = NodeEvaluationConfig {
        condition: EvaluationCondition::OnDemand,
        partitioned_output: true,
        ..Default::default()
    };
    let node = graph.create_node_with_config(config.clone());
    graph.get_entry_mut(node).unwrap().set_tombstoned(true);
    let wire = serde_json::to_value(&graph).unwrap();
    assert!(wire["arena"].get("definitions").is_none());
    assert_eq!(wire["arena"]["warm"][0]["tombstoned"], true);
    assert_eq!(
        wire["arena"]["warm"][0]["eval_config"]["condition"],
        "OnDemand"
    );
    let restored: SignalGraph = serde_json::from_value(wire).unwrap();
    assert_eq!(restored.node_eval_config(node).unwrap(), &config);
    assert!(restored.get_entry(node).unwrap().is_tombstoned());
}

#[test]
fn evaluation_writeback_preserves_shared_definition_pages() {
    let mut parent = SignalGraph::new();
    let node = parent.create_node();
    let (mut child, _) = parent.fork_persistent();
    let definitions = child.arena.definitions.page_identities();
    assert_eq!(definitions, parent.arena.definitions.page_identities());
    child
        .get_entry_mut(node)
        .unwrap()
        .set_aspect_version(AspectVersion::from_updates([(Aspect::new(0), 7)]));
    assert_eq!(child.arena.definitions.page_identities(), definitions);
    assert_eq!(
        parent
            .node_version_for_scope(node, Aspect::new(0), None)
            .unwrap(),
        0
    );
    assert_eq!(
        child
            .node_version_for_scope(node, Aspect::new(0), None)
            .unwrap(),
        7
    );

    let config = NodeEvaluationConfig {
        condition: EvaluationCondition::OnDemand,
        ..Default::default()
    };
    child
        .get_entry_mut(node)
        .unwrap()
        .set_eval_config(config.clone());
    assert_ne!(child.arena.definitions.page_identities(), definitions);
    assert_eq!(child.node_eval_config(node).unwrap(), &config);
    assert_eq!(
        parent.node_eval_config(node).unwrap(),
        &NodeEvaluationConfig::default()
    );
}
