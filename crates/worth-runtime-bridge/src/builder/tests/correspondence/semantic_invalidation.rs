use super::*;

#[test]
fn whole_aspect_set_and_clear_invalidate_a_field_dependency() {
    let mut graph = SignalGraph::new();
    let node = graph.node().build();
    let runtime = runtime(
        exact_mapping(),
        vec![registration(
            dependency("query:one"),
            vec![target(&graph, node)],
        )],
    );
    let TransitionOutcome::Success(correspondence) =
        runtime.install_semantic_correspondence(dependency("query:one"), &graph)
    else {
        panic!("field dependency correspondence should install")
    };
    let signal_target = correspondence.targets().next().unwrap();
    let aspect = signal_target.aspect();

    for kind in [
        AuthoritativeAspectChangeKind::WholeAspectSet,
        AuthoritativeAspectChangeKind::WholeAspectClear,
    ] {
        let before = graph.node_aspect_version(node).unwrap().get(aspect);
        let TransitionOutcome::Success(counters) = runtime
            .deliver_installed_correspondence_envelope(
                &correspondence,
                &mut graph,
                &whole_aspect_change_envelope(kind),
            )
        else {
            panic!("whole-aspect mutation should invalidate a field dependency")
        };
        assert_eq!(counters.truth_targets_admitted(), 1);
        assert_eq!(counters.signal_seeds_emitted(), 1);
        let prepared = counters
            .prepared_signal_invalidation()
            .expect("matched record-local truth prepares one scoped Signal seed");
        let regions = prepared.changed_regions().next().unwrap().as_slice();
        assert_eq!(regions.len(), 1);
        assert_eq!(
            regions[0].path().segments(),
            ["bridge-main", "relational-record:entity:0:1:1"]
        );
        assert_eq!(
            graph.node_aspect_version(node).unwrap().get(aspect),
            before + 1
        );
    }
}

#[test]
fn record_local_dependency_does_not_widen_an_unidentified_item() {
    let mut graph = SignalGraph::new();
    let node = graph.node().build();
    let runtime = runtime(
        exact_mapping(),
        vec![registration(
            dependency("query:one"),
            vec![target(&graph, node)],
        )],
    );
    let TransitionOutcome::Success(correspondence) =
        runtime.install_semantic_correspondence(dependency("query:one"), &graph)
    else {
        panic!("record-local dependency correspondence should install")
    };
    let aspect = correspondence.targets().next().unwrap().aspect();
    let before = graph.node_aspect_version(node).unwrap().get(aspect);
    let TransitionOutcome::Success(counters) = runtime.deliver_installed_correspondence_envelope(
        &correspondence,
        &mut graph,
        &unidentified_whole_aspect_envelope(),
    ) else {
        panic!("unmatched descriptive items are a successful no-op")
    };
    assert_eq!(counters.truth_targets_admitted(), 0);
    assert_eq!(counters.signal_seeds_emitted(), 0);
    assert_eq!(graph.node_aspect_version(node).unwrap().get(aspect), before);
}

#[test]
fn sealed_hierarchical_scopes_survive_field_correspondence_lowering() {
    let mut graph = SignalGraph::new();
    let node = graph.node().build();
    let runtime = runtime(
        exact_mapping(),
        vec![registration(
            dependency("query:one"),
            vec![target(&graph, node)],
        )],
    );
    let TransitionOutcome::Success(correspondence) =
        runtime.install_semantic_correspondence(dependency("query:one"), &graph)
    else {
        panic!("field correspondence should install")
    };
    let segments = [
        "bridge-main",
        "usd",
        "swaps",
        "book",
        "desk",
        "curve",
        "tenor",
        "5y",
    ];
    let aspect = correspondence.targets().next().unwrap().aspect();
    for depth in [1, 2, 4, 8] {
        let path = worth_signal::facade::ScopePath::new(
            segments[..depth]
                .iter()
                .map(|segment| (*segment).to_owned()),
        )
        .unwrap();
        let scope = worth_signal::facade::ChangedRegion::exact(path.clone());
        let before = graph.node_aspect_version(node).unwrap().get(aspect);
        let TransitionOutcome::Success(counters) = runtime
            .deliver_installed_correspondence_envelope(
                &correspondence,
                &mut graph,
                &field_change_envelope_with_scope(scope.clone()),
            )
        else {
            panic!("sealed field change should deliver at depth {depth}")
        };
        let regions = counters
            .prepared_signal_invalidation()
            .unwrap()
            .changed_regions()
            .next()
            .unwrap()
            .as_slice();
        assert_eq!(
            regions,
            &[scope],
            "Bridge delivery must preserve every ScopePath segment"
        );
        assert_eq!(regions[0].path(), &path);
        assert_eq!(regions[0].path().depth(), depth);
        assert_eq!(
            graph.node_aspect_version(node).unwrap().get(aspect),
            before + 1
        );
    }
}
