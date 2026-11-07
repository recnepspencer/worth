use worth_execution::{CancellationToken, ExecutionRequest, SerialMemoryBudget, SerialRequest};
use worth_signal::facade::durability::{SignalAuthoredImage, SignalAuthoredImageDenial};
use worth_signal::facade::{
    Aspect, AspectVersion, DependencyEdge, EvaluationContext, SignalError, SignalGraph,
    SignalRuntime,
};

fn request() -> SerialRequest {
    SerialRequest::from_memory(
        SerialMemoryBudget::new(Some(8 << 20)),
        CancellationToken::new(),
        None,
    )
}

#[test]
fn retiring_a_source_removes_declarations_after_evaluation_changes_the_read_set() {
    let funding = request();
    let execution = ExecutionRequest::serial(&funding);
    for discovered_only in [false, true] {
        let mut graph = SignalGraph::new();
        let source = graph.node().build();
        let retained = graph.node().build();
        let consumer = graph.node().on_demand().build();
        graph
            .set_dependencies(
                consumer,
                [
                    DependencyEdge::new(source, Aspect::new(0)),
                    DependencyEdge::new(retained, Aspect::new(0)),
                ],
            )
            .unwrap();
        if discovered_only {
            graph
                .set_evaluated_dependencies(
                    consumer,
                    [DependencyEdge::new(retained, Aspect::new(0))],
                )
                .unwrap();
        }
        // Both memory-only reconstruction doors must rebuild the declaration index.
        let encoded = serde_json::to_vec(&graph).unwrap();
        let decoded: SignalGraph = serde_json::from_slice(&encoded).unwrap();
        let rebound = graph
            .reconstitute_for_runtime_rebind()
            .unwrap()
            .into_parts()
            .0;
        for mut actual in [graph, decoded, rebound] {
            let mut reference = actual.clone();
            reference
                .set_dependencies(consumer, [DependencyEdge::new(retained, Aspect::new(0))])
                .unwrap();
            reference.unregister_node(source).unwrap();
            actual.unregister_node(source).unwrap();
            assert_eq!(
                actual.declared_dependencies_of(consumer).unwrap(),
                reference.declared_dependencies_of(consumer).unwrap()
            );
            assert_eq!(
                actual
                    .capture_authored_image(&execution)
                    .unwrap()
                    .as_bytes(),
                reference
                    .capture_authored_image(&execution)
                    .unwrap()
                    .as_bytes()
            );
            let reused = actual.create_node();
            assert_eq!(reused.index(), source.index());
            assert_ne!(reused, source);
            assert!(actual.declared_dependencies_of(reused).unwrap().is_empty());
            actual
                .set_dependencies(consumer, [DependencyEdge::new(reused, Aspect::new(0))])
                .unwrap();
            actual.unregister_node(reused).unwrap();
            assert!(actual
                .declared_dependencies_of(consumer)
                .unwrap()
                .is_empty());
        }
    }
}

#[test]
fn declaration_cycles_refuse_when_evaluation_has_removed_the_runtime_edge() {
    let mut graph = SignalGraph::new();
    let left = graph.create_node();
    let right = graph.create_node();
    graph
        .set_dependencies(left, [DependencyEdge::new(right, Aspect::new(0))])
        .unwrap();
    graph.set_evaluated_dependencies(left, []).unwrap();
    assert!(graph
        .set_dependencies(right, [DependencyEdge::new(left, Aspect::new(0))])
        .is_err());
    assert!(graph.declared_dependencies_of(right).unwrap().is_empty());
    assert!(graph.dependencies_of(right).unwrap().is_empty());
}

#[test]
fn authored_bytes_exclude_evaluated_edges_versions_and_node_state() {
    let funding = request();
    let execution = ExecutionRequest::serial(&funding);
    let mut graph = SignalGraph::new();
    let declared = graph.node().build();
    let discovered = graph.node().build();
    let consumer = graph.node().on_demand().build();
    graph
        .set_dependencies(
            consumer,
            [DependencyEdge::partition_detail(
                declared,
                Aspect::new(0),
                "orders",
                "price",
            )],
        )
        .unwrap();
    let authored = graph.capture_authored_image(&execution).unwrap();
    let mut runtime = SignalRuntime::build_for::<()>(graph);
    let evaluate = |view: &mut EvaluationContext<'_, ()>| -> Result<AspectVersion, SignalError> {
        if view.node() == consumer {
            view.read_aspect_version(discovered, Aspect::new(1))?;
        }
        Ok(AspectVersion::from_updates([(Aspect::new(1), 0xDEAD_BEEF)]))
    };
    assert_eq!(
        runtime
            .read(consumer, &(), &evaluate)
            .unwrap()
            .get(Aspect::new(1)),
        0xDEAD_BEEF
    );
    let graph = runtime.graph();
    assert_eq!(
        graph.declared_dependencies_of(consumer).unwrap()[0].source(),
        declared
    );
    assert_eq!(
        graph.dependencies_of(consumer).unwrap()[0].source(),
        discovered
    );
    let after_evaluation = graph.capture_authored_image(&execution).unwrap();
    assert_eq!(authored.as_bytes(), after_evaluation.as_bytes());
    assert!(!authored
        .as_bytes()
        .windows(8)
        .any(|part| part == 0xDEAD_BEEF_u64.to_le_bytes()));
    runtime
        .graph_mut()
        .set_dependencies(consumer, [DependencyEdge::new(discovered, Aspect::new(1))])
        .unwrap();
    assert_ne!(
        authored.as_bytes(),
        runtime
            .graph()
            .capture_authored_image(&execution)
            .unwrap()
            .as_bytes()
    );
    SignalAuthoredImage::readmit_bytes(&execution, authored.as_bytes()).unwrap();
}

#[test]
fn authored_image_has_explicit_empty_payload_and_refuses_unknown_versions() {
    let funding = request();
    let execution = ExecutionRequest::serial(&funding);
    let image = SignalGraph::new()
        .capture_authored_image(&execution)
        .unwrap();
    let bytes = image.as_bytes();
    assert_eq!(&bytes[..8], b"WSIGAUTH");
    assert_eq!(&bytes[8..12], &1_u32.to_le_bytes());
    assert_eq!(&bytes[16..24], &8_u64.to_le_bytes());
    assert_eq!(
        &bytes[56..],
        &[0; 8],
        "zero slots and zero allocator free entries"
    );
    let mut future = bytes.to_vec();
    future[8..12].copy_from_slice(&2_u32.to_le_bytes());
    assert!(
        matches!(SignalAuthoredImage::readmit_bytes(&execution, &future),
        Err(SignalAuthoredImageDenial::UnsupportedVersion(version)) if version.produced().get() == 2)
    );
}

#[test]
fn checksum_and_exact_length_cover_the_authored_frame() {
    let funding = request();
    let execution = ExecutionRequest::serial(&funding);
    let mut graph = SignalGraph::new();
    graph.node().build();
    let image = graph.capture_authored_image(&execution).unwrap();
    let mut changed = image.as_bytes().to_vec();
    *changed.last_mut().unwrap() ^= 1;
    assert!(matches!(
        SignalAuthoredImage::readmit_bytes(&execution, &changed),
        Err(SignalAuthoredImageDenial::ChecksumMismatch)
    ));
    changed.push(0);
    assert!(matches!(
        SignalAuthoredImage::readmit_bytes(&execution, &changed),
        Err(SignalAuthoredImageDenial::Malformed)
    ));
}

#[test]
fn definition_policy_changes_are_source_even_when_cached_versions_match() {
    let funding = request();
    let execution = ExecutionRequest::serial(&funding);
    let mut graph = SignalGraph::new();
    let node = graph.node().build();
    let before = graph.capture_authored_image(&execution).unwrap();
    let mut changed = SignalGraph::new();
    let changed_node = changed
        .node()
        .custom_condition("retained-program-condition")
        .build();
    assert_eq!(node, changed_node);
    assert_eq!(
        graph.node_aspect_version(node).unwrap(),
        changed.node_aspect_version(changed_node).unwrap()
    );
    let after = changed.capture_authored_image(&execution).unwrap();
    assert_ne!(before.as_bytes(), after.as_bytes());
    assert!(after
        .as_bytes()
        .windows(26)
        .any(|part| part == b"retained-program-condition"));
    SignalAuthoredImage::readmit_bytes(&execution, after.as_bytes()).unwrap();
}

#[test]
fn memory_snapshot_keeps_declared_edges_separate_from_discovered_topology() {
    let mut graph = SignalGraph::new();
    let source = graph.node().build();
    let discovered = graph.node().build();
    let consumer = graph.node().build();
    graph
        .set_dependencies(consumer, [DependencyEdge::new(source, Aspect::new(0))])
        .unwrap();
    graph
        .set_evaluated_dependencies(consumer, [DependencyEdge::new(discovered, Aspect::new(1))])
        .unwrap();
    let restored: SignalGraph =
        serde_json::from_slice(&serde_json::to_vec(&graph).unwrap()).unwrap();
    assert_eq!(
        restored.declared_dependencies_of(consumer).unwrap()[0].source(),
        source
    );
    assert_eq!(
        restored.dependencies_of(consumer).unwrap()[0].source(),
        discovered
    );
}

#[test]
fn bulk_declarations_are_captured_and_rejected_batches_preserve_the_source() {
    use worth_signal::facade::adapters::DependencyBatchEdit;
    let funding = request();
    let execution = ExecutionRequest::serial(&funding);
    let mut graph = SignalGraph::new();
    let source = graph.node().build();
    let consumer = graph.node().build();
    graph
        .apply_dependency_batch_edit(&DependencyBatchEdit::from_pairs([(
            consumer,
            vec![DependencyEdge::new(source, Aspect::new(0))],
        )]))
        .unwrap();
    assert_eq!(
        graph.declared_dependencies_of(consumer).unwrap()[0].source(),
        source
    );
    let before = graph.capture_authored_image(&execution).unwrap();
    let rejected = DependencyBatchEdit::from_pairs([
        (source, vec![DependencyEdge::new(consumer, Aspect::new(0))]),
        (consumer, vec![DependencyEdge::new(source, Aspect::new(0))]),
    ]);
    assert!(graph.apply_dependency_batch_edit(&rejected).is_err());
    assert_eq!(
        before.as_bytes(),
        graph.capture_authored_image(&execution).unwrap().as_bytes()
    );
}
