use crate::data::trace::RuntimeArtifactState;
use crate::facade::{
    mark_dirty, ArtifactSemanticBoundary, AuthorityPolicy, CanonicalChangedRegions,
    CausalDisposition, CausalityMetadata, ConditionDecision, ContextRequirement,
    EvaluationCondition, MemoizedResultOrigin, NodeId, PartitionScopeSet, PartitionSubscription,
    RetainedDiagnosticArtifact, ReuseBasis, ReuseBoundaryContext, ReuseBoundaryProof,
    ReuseCertificationRecord, ReuseCrossing, ReuseSemanticRegionIdentity, ReuseSource, SignalGraph,
    UpstreamCause, VersionComparatorPolicy,
};
use crate::tests::support::{
    evaluate, evaluate_on_demand, version_ab, GraphDependencyBatchExt, ASPECT_A,
};

#[test]
fn explain_reports_condition_deferred_for_on_demand_nodes() {
    let mut graph = SignalGraph::new();
    let source = graph.node().build();
    let dependent = graph.node().on_demand().build();
    graph
        .append_dependency(dependent, source, ASPECT_A)
        .unwrap();

    let mut source_v1 = |_id: NodeId, _graph: &SignalGraph| Ok(version_ab(1, 0));
    let mut source_v2 = |_id: NodeId, _graph: &SignalGraph| Ok(version_ab(2, 0));
    let mut dependent_compute = |_id: NodeId, _graph: &SignalGraph| Ok(version_ab(10, 0));

    evaluate(&mut graph, source, &mut source_v1).unwrap();
    evaluate_on_demand(&mut graph, dependent, &mut dependent_compute).unwrap();
    mark_dirty(&mut graph, source, ASPECT_A).unwrap();
    evaluate(&mut graph, source, &mut source_v2).unwrap();
    evaluate(&mut graph, dependent, &mut dependent_compute).unwrap();

    let explanation = graph.observe().explain(dependent).unwrap();
    assert!(explanation
        .upstream
        .iter()
        .any(|cause| matches!(cause, UpstreamCause::ConditionDeferred { source: deferred, aspect, condition: EvaluationCondition::OnDemand, decision: ConditionDecision::Deferred, .. } if *deferred == source && *aspect == ASPECT_A)));
}

#[test]
fn explain_reports_missing_snapshot_and_dependency_removed() {
    let mut graph = SignalGraph::new();
    let source = graph.node().build();
    let dependent = graph.node().build();
    let mut source_compute = |_id: NodeId, _graph: &SignalGraph| Ok(version_ab(1, 0));
    evaluate(&mut graph, source, &mut source_compute).unwrap();

    graph
        .append_dependency(dependent, source, ASPECT_A)
        .unwrap();
    let missing_snapshot = graph.observe().explain(dependent).unwrap();
    assert!(missing_snapshot
        .upstream
        .iter()
        .any(|cause| matches!(cause, UpstreamCause::MissingSnapshot { source: missing, aspect, current_version: Some(1), .. } if *missing == source && *aspect == ASPECT_A)));
    assert!(missing_snapshot.causal_links.iter().any(|link| {
        matches!(link.disposition, CausalDisposition::Conservative)
            && matches!(
                link.kind,
                crate::logic::explain::CausalLinkKind::MissingSnapshot
            )
    }));

    let mut dependent_compute = |_id: NodeId, _graph: &SignalGraph| Ok(version_ab(10, 0));
    evaluate(&mut graph, dependent, &mut dependent_compute).unwrap();
    graph.drop_dependency(dependent, source, ASPECT_A).unwrap();

    let removed = graph.observe().explain(dependent).unwrap();
    assert!(removed
        .upstream
        .iter()
        .any(|cause| matches!(cause, UpstreamCause::DependencyRemoved { source: removed_source, aspect, cached_version: 1, .. } if *removed_source == source && *aspect == ASPECT_A)));
}

#[test]
fn explanation_surfaces_causality_and_trace_summary() {
    let mut graph = SignalGraph::new();
    let node = graph.node().build();
    graph
        .set_causality(
            node,
            Some(CausalityMetadata {
                kind: "bridge".to_string(),
                fields: [("commit".to_string(), "42".to_string())]
                    .into_iter()
                    .collect(),
            }),
        )
        .unwrap();

    let mut compute = |_id: NodeId, _graph: &SignalGraph| Ok(version_ab(1, 0));
    evaluate(&mut graph, node, &mut compute).unwrap();

    let explanation = graph.observe().explain(node).unwrap();
    assert_eq!(explanation.causality.as_ref().unwrap().kind, "bridge");
    assert!(explanation.historical_artifact_record.is_some());
    assert!(format!("{explanation}").contains("Causality: bridge"));
}

#[test]
fn explanation_surfaces_retained_reuse_certification() {
    let mut graph = SignalGraph::new();
    let node = graph.node().build();
    {
        let mut entry = graph.get_entry_mut(node).unwrap();
        let reuse_boundary_context = ReuseBoundaryContext {
            topology_regime: 1,
            tolerance_regime: VersionComparatorPolicy::Exact,
            semantic_region: ReuseSemanticRegionIdentity::new(
                node,
                false,
                Vec::new(),
                ContextRequirement::None,
            ),
            authority_policy: AuthorityPolicy::SpeculativeThenReconcile,
            artifact_family: None,
            structural_dependency_basis: crate::data::dependency::DependencySnapshotId::EMPTY,
            partition_region_basis: PartitionScopeSet::default(),
            strategy_detail: crate::data::reuse::ReuseStrategyBoundaryContext::None,
        };
        let mut runtime = RuntimeArtifactState::default();
        runtime.hot_mut().recomputed = false;
        runtime.warm_mut().memoized_origin = MemoizedResultOrigin::MemoizedFromCache;
        runtime.warm_mut().reuse_origin = crate::data::reuse::ReuseOrigin::MemoizedArtifactReuse;
        runtime.warm_mut().reuse_basis =
            crate::data::trace::ReuseOperationalBasis::new(ReuseBasis::strategy(
                crate::data::reuse::ReuseStrategy::MemoizedArtifactReuse,
                ReuseSource::MemoizedArtifact,
                ReuseCrossing::None,
            ));
        runtime.warm_mut().reuse_boundary_authority = Some(reuse_boundary_context.authority());
        entry.set_runtime_artifact_state(Some(runtime));
        entry.set_retained_diagnostic_artifact(Some(RetainedDiagnosticArtifact {
            changed_regions: CanonicalChangedRegions::default(),
            labels: Vec::new(),
            keyed_family: None,
            keyed_key: None,
            reuse_certification: Some(ReuseCertificationRecord {
                strategy: crate::data::reuse::ReuseStrategy::MemoizedArtifactReuse,
                origin: crate::data::reuse::ReuseOrigin::MemoizedArtifactReuse,
                source: ReuseSource::MemoizedArtifact,
                crossing: ReuseCrossing::None,
                proofs: vec![ReuseBoundaryProof {
                    boundary: ArtifactSemanticBoundary::TopologyRegime,
                    satisfied: true,
                }],
            }),
            reuse_boundary_context: Some(reuse_boundary_context),
        }));
    }

    let explanation = graph.observe().explain(node).unwrap();
    assert_eq!(
        explanation.reuse_basis,
        Some(ReuseBasis::strategy(
            crate::data::reuse::ReuseStrategy::MemoizedArtifactReuse,
            ReuseSource::MemoizedArtifact,
            ReuseCrossing::None,
        ))
    );
    assert_eq!(
        explanation
            .reuse_certification
            .as_ref()
            .map(|record| record.proofs.len()),
        Some(1)
    );
    assert!(format!("{explanation}").contains("Reuse certification proofs: 1"));
}

#[test]
fn reuse_boundary_detail_lives_in_cold_retained_lane_while_hot_runtime_keeps_compact_authority() {
    let mut graph = SignalGraph::new();
    let node = graph.node().build();
    let reuse_boundary_context = ReuseBoundaryContext {
        topology_regime: 7,
        tolerance_regime: VersionComparatorPolicy::Exact,
        semantic_region: ReuseSemanticRegionIdentity::new(
            node,
            true,
            vec![PartitionSubscription::whole_partition("wing")],
            ContextRequirement::RelationalSnapshot,
        ),
        authority_policy: AuthorityPolicy::SpeculativeThenReconcile,
        artifact_family: Some(crate::data::reuse::ArtifactFamilyId::new("projection")),
        structural_dependency_basis: crate::data::dependency::DependencySnapshotId::EMPTY,
        partition_region_basis: PartitionScopeSet::new([PartitionSubscription::whole_partition(
            "wing",
        )]),
        strategy_detail: crate::data::reuse::ReuseStrategyBoundaryContext::CrossIdentity {
            persistent_correspondence:
                crate::data::reuse::PersistentCorrespondenceEvidence::LineageBackedMapping(
                    "lineage-map:wing-a->wing-b".to_string(),
                ),
        },
    };
    {
        let mut entry = graph.get_entry_mut(node).unwrap();
        let mut runtime = RuntimeArtifactState::default();
        runtime.warm_mut().reuse_origin =
            crate::data::reuse::ReuseOrigin::CrossIdentityPersistentReuse;
        runtime.warm_mut().reuse_basis =
            crate::data::trace::ReuseOperationalBasis::new(ReuseBasis::strategy(
                crate::data::reuse::ReuseStrategy::CrossIdentityPersistentMatch,
                ReuseSource::PersistentCorrespondence,
                ReuseCrossing::PersistentIdentityBoundary,
            ));
        runtime.warm_mut().reuse_boundary_authority = Some(reuse_boundary_context.authority());
        entry.set_runtime_artifact_state(Some(runtime));
        entry.set_retained_diagnostic_artifact(Some(RetainedDiagnosticArtifact {
            changed_regions: CanonicalChangedRegions::default(),
            labels: Vec::new(),
            keyed_family: None,
            keyed_key: None,
            reuse_certification: None,
            reuse_boundary_context: Some(reuse_boundary_context.clone()),
        }));
    }

    let runtime = graph
        .observe()
        .runtime_artifact_state(node)
        .unwrap()
        .cloned()
        .expect("runtime artifact");
    assert_eq!(
        runtime
            .reuse_boundary_authority()
            .and_then(|authority| authority.persistent_correspondence_kind()),
        Some(crate::data::reuse::PersistentCorrespondenceKind::LineageBackedMapping)
    );

    let trace_summary = graph
        .observe()
        .materialize()
        .materialize_trace_summary(node)
        .unwrap()
        .expect("trace summary");
    assert_eq!(
        trace_summary.reuse_boundary_context,
        Some(reuse_boundary_context)
    );
}
