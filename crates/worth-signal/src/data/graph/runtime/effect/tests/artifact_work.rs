use super::*;
use crate::data::graph::SignalGraph;
use crate::data::output::{
    ArtifactContinuityToken, CanonicalChangedRegions, OutputIdentity, PartitionSubscription,
};
use crate::data::output_equivalence::OutputEquivalencePolicy;
use crate::data::proof::PartitionScopeSet;
use crate::data::retained_storage::RetainedStoragePreparation as Work;
use crate::logic::evaluation::EvaluationWork;

#[test]
fn artifact_scope_preparation_matches_library_canonicalization_with_exact_and_short_work() {
    let mut graph = SignalGraph::new();
    graph.set_runtime_policy(crate::facade::SignalRuntimePolicy::forensic());
    let node = graph.create_node();
    for count in 0..24 {
        let regions: Vec<_> = (0..count)
            .rev()
            .map(|n| ChangedRegion::new(format!("p-{}", n % 5)).with_detail(format!("d-{}", n % 3)))
            .collect();
        let canonical = CanonicalChangedRegions::from_slice(&regions);
        let mut effect = test_effect_with_labels(vec!["retained".to_owned()]);
        effect.operational.node = node;
        effect.diagnostics =
            DiagnosticEnvelope::from_parts(None, None, regions, vec!["retained".to_owned()]);
        let comparison = graph
            .compare_effect(
                &effect,
                None,
                None,
                OutputEquivalencePolicy::ExactAspectVersion,
                &mut EvaluationWork::Ordinary,
            )
            .unwrap();
        let mut measured = Work::new(10_000_000);
        let artifact = graph
            .build_effect_artifact_write(
                &effect,
                None,
                None,
                comparison,
                &mut EvaluationWork::Conditional(&mut measured),
            )
            .unwrap()
            .unwrap();
        assert_eq!(
            artifact.cold_intent.as_ref().unwrap().changed_regions,
            canonical
        );
        assert_eq!(
            artifact.runtime.as_ref().unwrap().hot().changed_scopes,
            crate::data::trace::CompactChangedScopeProof::new(
                PartitionScopeSet::from_changed_regions(&canonical)
            )
        );
        let cost = measured.visits();
        for available in [cost - 1, cost] {
            let mut work = Work::new(cost + 7);
            work.reserve_visits(cost + 7 - available).unwrap();
            let result = graph.build_effect_artifact_write(
                &effect,
                None,
                None,
                comparison,
                &mut EvaluationWork::Conditional(&mut work),
            );
            if available == cost {
                assert_eq!(result.unwrap().unwrap(), artifact);
            } else {
                assert!(matches!(
                    result,
                    Err(crate::data::error::SignalError::ConditionalEvaluationWorkExhausted { .. })
                ));
            }
            assert!(!graph.node_runtime_artifact_state_present(node).unwrap());
        }
    }
}

#[test]
fn warm_artifact_charges_selected_prior_tokens_and_custom_reuse_payload() {
    let mut graph = SignalGraph::new();
    let node = graph.create_node();
    let mut effect = test_effect_with_labels(Vec::new());
    effect.operational.node = node;
    effect.operational.verdict = EvaluationVerdict::Suppressed {
        reason: crate::logic::evaluation::SuppressionReason::ComparatorMatch,
    };
    let comparison = graph
        .compare_effect(
            &effect,
            None,
            None,
            OutputEquivalencePolicy::ExactAspectVersion,
            &mut EvaluationWork::Ordinary,
        )
        .unwrap();
    let mut short = Work::new(1_000_000);
    graph
        .build_effect_artifact_write(
            &effect,
            None,
            None,
            comparison,
            &mut EvaluationWork::Conditional(&mut short),
        )
        .unwrap();
    let identity = OutputIdentity::new("prior-λ".repeat(2_000));
    let continuity = ArtifactContinuityToken::new("prior-continuity".repeat(2_000));
    effect.operational.reuse_basis.artifact_family_basis = Some(
        crate::data::reuse::ArtifactFamilyId::new("family".repeat(2_000)),
    );
    effect.operational.reuse_boundary_authority.tolerance_regime =
        crate::data::comparator::VersionComparatorPolicy::Custom {
            key: "comparator".repeat(2_000),
        };
    assert!(matches!(
        graph.build_effect_artifact_write(
            &effect,
            Some(&identity),
            Some(&continuity),
            comparison,
            &mut EvaluationWork::Conditional(&mut Work::new(short.visits()))
        ),
        Err(crate::data::error::SignalError::ConditionalEvaluationWorkExhausted { .. })
    ));
    let mut work = Work::new(10_000_000);
    let artifact = graph
        .build_effect_artifact_write(
            &effect,
            Some(&identity),
            Some(&continuity),
            comparison,
            &mut EvaluationWork::Conditional(&mut work),
        )
        .unwrap()
        .unwrap();
    let runtime = artifact.runtime.unwrap();
    assert_eq!(runtime.warm().output_identity.as_ref(), Some(&identity));
    assert_eq!(runtime.warm().continuity_token.as_ref(), Some(&continuity));
    assert!(work.visits() >= identity.as_str().len() + continuity.as_str().len());
}

#[test]
fn cold_detail_copies_every_owned_scope_and_omit_policy_does_no_payload_work() {
    let mut effect = test_effect_with_labels(vec!["label".repeat(2_000)]);
    effect.operational.reuse_basis.strategy = Some(ReuseStrategy::PartialArtifactSplicing);
    let text = "detail-λ".repeat(2_000);
    let scope = PartitionSubscription::partition_and_detail(text.clone(), text.clone());
    effect.runtime_metadata.reuse_boundary_detail = Some(ReuseBoundaryContext {
        topology_regime: 1,
        tolerance_regime: crate::data::comparator::VersionComparatorPolicy::Custom {
            key: text.clone(),
        },
        semantic_region: ReuseSemanticRegionIdentity::new(
            effect.operational.node,
            true,
            vec![scope.clone()],
            crate::data::node::ContextRequirement::None,
        ),
        authority_policy: crate::data::performance::AuthorityPolicy::SpeculativeThenReconcile,
        artifact_family: Some(crate::data::reuse::ArtifactFamilyId::new(text.clone())),
        structural_dependency_basis: crate::data::dependency::DependencySnapshotId::EMPTY,
        partition_region_basis: PartitionScopeSet::new([scope.clone()]),
        strategy_detail: crate::data::reuse::ReuseStrategyBoundaryContext::PartialArtifactSplice {
            composition_regions: PartitionScopeSet::new([scope]),
        },
    });
    let scopes = PartitionScopeSet::default();
    let retention = RetentionBudget::development();
    let mut work = Work::new(10_000_000);
    let intent = build_cold_artifact_intent(
        &effect,
        &retention,
        &scopes,
        &mut EvaluationWork::Conditional(&mut work),
    )
    .unwrap()
    .unwrap();
    assert_eq!(
        intent.reuse_boundary_context.as_ref(),
        effect.reuse_boundary_detail()
    );
    assert!(
        work.visits() >= 8 * text.len(),
        "three two-string scope copies, custom key and family"
    );
    let mut denied = Work::new(8 * text.len() - 1);
    assert!(matches!(
        build_cold_artifact_intent(
            &effect,
            &retention,
            &scopes,
            &mut EvaluationWork::Conditional(&mut denied)
        ),
        Err(crate::data::error::SignalError::ConditionalEvaluationWorkExhausted { .. })
    ));
    let omit = RetentionBudget {
        explanation_retention: ArtifactRetentionPolicy::Omit,
        provenance_retention: ArtifactRetentionPolicy::Omit,
        ..retention
    };
    let mut zero = Work::new(0);
    assert!(build_cold_artifact_intent(
        &effect,
        &omit,
        &scopes,
        &mut EvaluationWork::Conditional(&mut zero)
    )
    .unwrap()
    .is_none());
    assert_eq!(zero.visits(), 0);
}

#[test]
fn canonical_region_value_owner_rejects_duplicates_and_reversed_order() {
    assert!(CanonicalChangedRegions::from_canonical_regions(vec![
        ChangedRegion::new("b"),
        ChangedRegion::new("a")
    ])
    .is_none());
    assert!(CanonicalChangedRegions::from_canonical_regions(vec![
        ChangedRegion::new("a"),
        ChangedRegion::new("a")
    ])
    .is_none());
    assert!(CanonicalChangedRegions::from_canonical_regions(vec![
        ChangedRegion::new("a"),
        ChangedRegion::new("b")
    ])
    .is_some());
}
