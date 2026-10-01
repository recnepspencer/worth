use crate::facade::*;
use crate::tests::support::*;

#[test]
fn transitive_maybe_stale_nodes_do_not_carry_foreign_source_partition_scopes() {
    let mut graph = SignalGraph::new();
    let source = graph.node().partitioned_output().build();
    let direct = graph.node().build();
    let transitive = graph.node().build();
    graph
        .append_partition_detail_dependency(direct, source, ASPECT_A, "wing", "rib-12")
        .unwrap();
    graph
        .append_dependency(transitive, direct, ASPECT_A)
        .unwrap();

    mark_dirty_with_regions(
        &mut graph,
        source,
        ASPECT_A,
        &[ChangedRegion::new("wing").with_detail("rib-12")],
    )
    .unwrap();

    let transitive_entry = graph.get_entry(transitive).unwrap();
    assert_eq!(graph.get_state(direct).unwrap(), NodeState::Dirty);
    assert_eq!(graph.get_state(transitive).unwrap(), NodeState::Dirty);
    assert!(
        transitive_entry.get_dirty_partition_scopes().is_empty(),
        "transitive node should validate from its direct upstream trace, not copied source scopes"
    );
}

#[test]
fn local_scope_explanations_mark_untouched_partition_evidence_as_discarded() {
    let mut graph = SignalGraph::new();
    let source = graph.node().partitioned_output().build();
    let dependent = graph.node().build();
    graph
        .append_partition_detail_dependency(dependent, source, ASPECT_A, "wing", "rib-12")
        .unwrap();

    let bootstrap = graph
        .build_evaluation_plan(&[source, dependent], EvaluationRequestMode::ForceOnDemand)
        .unwrap();
    graph
        .execute_prepared_plan(&bootstrap, &(), &|ctx| {
            let result = if ctx.node() == dependent {
                let version = ctx.read_partitioned_aspect_version(
                    source,
                    ASPECT_A,
                    PartitionSubscription::partition_and_detail("wing", "rib-12"),
                )?;
                ctx.finish(NodeEvaluationResult::from_version(version))
            } else {
                ctx.finish(NodeEvaluationResult::from_version(version_ab(1, 0)))
            };
            Ok(result)
        })
        .unwrap();

    mark_dirty_with_regions(
        &mut graph,
        source,
        ASPECT_A,
        &[ChangedRegion::new("wing").with_detail("rib-13")],
    )
    .unwrap();
    let source_plan = graph
        .build_evaluation_plan(&[source], EvaluationRequestMode::Default)
        .unwrap();
    graph
        .execute_prepared_plan(&source_plan, &(), &|ctx| {
            Ok(ctx.finish(
                NodeEvaluationResult::from_version(version_ab(2, 0))
                    .with_changed_region(ChangedRegion::new("wing").with_detail("rib-13")),
            ))
        })
        .unwrap();

    let explanation = graph.observe().explain(dependent).unwrap();
    assert!(explanation.causal_links.iter().any(|link| {
        matches!(
            link.kind,
            crate::logic::explain::CausalLinkKind::ScopeUntouched
        ) && matches!(link.scope.kind, ScopeProvenanceKind::Discarded)
    }));
}

#[test]
fn broader_partition_validation_scopes_report_translated_upstream_region_evidence() {
    let mut graph = SignalGraph::new();
    let source = graph.node().partitioned_output().build();
    let dependent = graph.node().build();
    graph
        .append_partition_dependency(dependent, source, ASPECT_A, "wing")
        .unwrap();

    let bootstrap = graph
        .build_evaluation_plan(&[source, dependent], EvaluationRequestMode::ForceOnDemand)
        .unwrap();
    graph
        .execute_prepared_plan(&bootstrap, &(), &|ctx| {
            let result = if ctx.node() == dependent {
                let version = ctx.read_partitioned_aspect_version(
                    source,
                    ASPECT_A,
                    PartitionSubscription::whole_partition("wing"),
                )?;
                ctx.finish(NodeEvaluationResult::from_version(version))
            } else {
                ctx.finish(NodeEvaluationResult::from_version(version_ab(1, 0)))
            };
            Ok(result)
        })
        .unwrap();

    mark_dirty_with_regions(
        &mut graph,
        source,
        ASPECT_A,
        &[ChangedRegion::new("wing").with_detail("rib-13")],
    )
    .unwrap();
    let source_plan = graph
        .build_evaluation_plan(&[source], EvaluationRequestMode::Default)
        .unwrap();
    graph
        .execute_prepared_plan(&source_plan, &(), &|ctx| {
            Ok(ctx.finish(
                NodeEvaluationResult::from_version(version_ab(2, 0))
                    .with_changed_region(ChangedRegion::new("wing").with_detail("rib-13")),
            ))
        })
        .unwrap();

    let explanation = graph.observe().explain(dependent).unwrap();
    assert!(explanation.causal_links.iter().any(|link| {
        matches!(link.kind, crate::logic::explain::CausalLinkKind::Changed)
            && matches!(link.scope.kind, ScopeProvenanceKind::Translated)
            && link
                .scope
                .source_scope
                .as_ref()
                .is_some_and(|scope| scope.path().segments() == ["wing", "rib-13"])
            && link
                .scope
                .validation_scope
                .as_ref()
                .is_some_and(|scope| scope.path().segments() == ["wing"])
    }));
}
