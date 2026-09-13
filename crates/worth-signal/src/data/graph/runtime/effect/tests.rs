use super::cold_artifact::build_cold_artifact_intent;
use super::vocabulary::record_reuse_telemetry;
use crate::data::aspect::AspectVersion;
use crate::data::dependency::{
    CommittedSnapshotUpdate, DependencySnapshot, ReplacementSnapshotUpdate,
    SharedDependencySnapshot, SnapshotDeltaRecord,
};
use crate::data::handle::NodeId;
use crate::data::output::ChangedRegion;
use crate::data::output::{MemoizedResultOrigin, OutputChange};
use crate::data::reuse::{
    ArtifactSemanticBoundary, ReuseBasis, ReuseBoundaryContext, ReuseBoundaryProof,
    ReuseCertificationRecord, ReuseCrossing, ReuseOrigin, ReuseSemanticRegionIdentity, ReuseSource,
    ReuseStrategy,
};
use crate::data::telemetry::RuntimeTelemetry;
use crate::diagnostics::policy::{ArtifactRetentionPolicy, RetentionBudget};
use crate::logic::evaluation::{
    DiagnosticEnvelope, EffectRuntimeMetadata, EvaluationEffect, EvaluationVerdict,
    OperationalEffect,
};

pub(super) fn test_effect_with_labels(labels: Vec<String>) -> EvaluationEffect {
    let node = NodeId::new(0, 0);
    EvaluationEffect {
        operational: OperationalEffect {
            node,
            verdict: EvaluationVerdict::Recomputed,
            aspect_version: AspectVersion::zero(),
            changed_aspect_regions: Vec::new(),
            output_change: OutputChange::Replaced,
            reuse_basis: ReuseBasis::strategy(
                ReuseStrategy::MemoizedArtifactReuse,
                ReuseSource::MemoizedArtifact,
                ReuseCrossing::None,
            ),
            reuse_origin: ReuseOrigin::MemoizedArtifactReuse,
            reuse_boundary_authority: ReuseBoundaryContext {
                topology_regime: 1,
                tolerance_regime: crate::data::comparator::VersionComparatorPolicy::Exact,
                semantic_region: ReuseSemanticRegionIdentity::new(
                    node,
                    false,
                    Vec::new(),
                    crate::data::node::ContextRequirement::None,
                ),
                authority_policy:
                    crate::data::performance::AuthorityPolicy::SpeculativeThenReconcile,
                artifact_family: None,
                structural_dependency_basis: crate::data::dependency::DependencySnapshotId::EMPTY,
                partition_region_basis: Default::default(),
                strategy_detail: crate::data::reuse::ReuseStrategyBoundaryContext::None,
            }
            .authority(),
            dependency_snapshot_update: CommittedSnapshotUpdate::Replace(
                ReplacementSnapshotUpdate::from_snapshot(DependencySnapshot::empty()),
            ),
            snapshot_delta: SnapshotDeltaRecord::between(
                node,
                &DependencySnapshot::empty(),
                &SharedDependencySnapshot::empty(),
            ),
            meaningful_input_changes: 0,
        },
        diagnostics: DiagnosticEnvelope::from_parts(
            Some("artifact".into()),
            Some("continuity".into()),
            vec![ChangedRegion::new("wing").with_detail("rib-12")],
            labels,
        ),
        runtime_metadata: EffectRuntimeMetadata::default(),
    }
}

#[test]
fn contradictory_unchanged_output_is_rejected_before_graph_mutation() {
    let mut graph = crate::data::graph::SignalGraph::new();
    let node = graph.create_node();
    let mut effect = test_effect_with_labels(Vec::new());
    effect.operational.node = node;
    effect.operational.aspect_version =
        AspectVersion::from_updates([(crate::data::aspect::Aspect::new(1), 1)]);
    effect.operational.output_change = OutputChange::Unchanged;

    let error = graph
        .compare_effect(
            &effect,
            None,
            None,
            crate::data::output_equivalence::OutputEquivalencePolicy::ExactAspectVersion,
            &mut crate::logic::evaluation::EvaluationWork::Ordinary,
        )
        .expect_err("contradictory output must fail before apply");

    assert!(error
        .to_string()
        .contains("output commit contract violation"));
    assert_eq!(
        graph.node_aspect_version(node).unwrap(),
        AspectVersion::zero()
    );
}

#[test]
fn retained_reuse_certification_increments_cold_materialization_counter() {
    let mut telemetry = RuntimeTelemetry::default();
    let node = NodeId::new(0, 0);
    let effect = EvaluationEffect {
        operational: OperationalEffect {
            node,
            verdict: EvaluationVerdict::Suppressed {
                reason: crate::logic::evaluation::SuppressionReason::ComparatorMatch,
            },
            aspect_version: AspectVersion::zero(),
            changed_aspect_regions: Vec::new(),
            output_change: OutputChange::Unchanged,
            reuse_basis: ReuseBasis::strategy(
                ReuseStrategy::MemoizedArtifactReuse,
                ReuseSource::MemoizedArtifact,
                ReuseCrossing::None,
            ),
            reuse_origin: ReuseOrigin::MemoizedArtifactReuse,
            reuse_boundary_authority: ReuseBoundaryContext {
                topology_regime: 1,
                tolerance_regime: crate::data::comparator::VersionComparatorPolicy::Exact,
                semantic_region: ReuseSemanticRegionIdentity::new(
                    node,
                    false,
                    Vec::new(),
                    crate::data::node::ContextRequirement::None,
                ),
                authority_policy:
                    crate::data::performance::AuthorityPolicy::SpeculativeThenReconcile,
                artifact_family: None,
                structural_dependency_basis: crate::data::dependency::DependencySnapshotId::EMPTY,
                partition_region_basis: Default::default(),
                strategy_detail: crate::data::reuse::ReuseStrategyBoundaryContext::None,
            }
            .authority(),
            dependency_snapshot_update: CommittedSnapshotUpdate::Replace(
                ReplacementSnapshotUpdate::from_snapshot(DependencySnapshot::empty()),
            ),
            snapshot_delta: SnapshotDeltaRecord::between(
                node,
                &DependencySnapshot::empty(),
                &SharedDependencySnapshot::empty(),
            ),
            meaningful_input_changes: 2,
        },
        diagnostics: None,
        runtime_metadata: EffectRuntimeMetadata {
            memoized_origin: MemoizedResultOrigin::MemoizedFromCache,
            recomputed: false,
            keyed_context: None,
            causality: None,
            reuse_certification: Some(ReuseCertificationRecord {
                strategy: ReuseStrategy::MemoizedArtifactReuse,
                origin: ReuseOrigin::MemoizedArtifactReuse,
                source: ReuseSource::MemoizedArtifact,
                crossing: ReuseCrossing::None,
                proofs: vec![ReuseBoundaryProof {
                    boundary: ArtifactSemanticBoundary::TopologyRegime,
                    satisfied: true,
                }],
            }),
            reuse_boundary_detail: None,
            previous_artifact_warm: None,
        },
    };

    record_reuse_telemetry(&mut telemetry, &effect);

    assert_eq!(telemetry.evaluation.memoized_reuse_count, 1);
    assert_eq!(
        telemetry
            .evaluation
            .reuse_cold_certification_materialization_count,
        1
    );
    assert_eq!(telemetry.evaluation.reuse_dependency_comparison_breadth, 2);
}

#[test]
fn cold_artifact_intent_is_bypassed_under_omit_policy() {
    let effect = test_effect_with_labels(vec![
        "alpha".to_string(),
        "beta".to_string(),
        "gamma".to_string(),
    ]);
    let retention = RetentionBudget {
        explanation_retention: ArtifactRetentionPolicy::Omit,
        provenance_retention: ArtifactRetentionPolicy::Omit,
        ..RetentionBudget::operational()
    };

    assert!(build_cold_artifact_intent(
        &effect,
        &retention,
        &crate::data::proof::PartitionScopeSet::default(),
        &mut crate::logic::evaluation::EvaluationWork::Ordinary
    )
    .unwrap()
    .is_none());
}

#[test]
fn cold_artifact_intent_caps_label_count() {
    let effect = test_effect_with_labels(vec![
        "a".to_string(),
        "b".to_string(),
        "c".to_string(),
        "d".to_string(),
        "e".to_string(),
        "f".to_string(),
    ]);
    let retention = RetentionBudget {
        explanation_retention: ArtifactRetentionPolicy::Retain,
        provenance_retention: ArtifactRetentionPolicy::Retain,
        ..RetentionBudget::development()
    };

    let intent = build_cold_artifact_intent(
        &effect,
        &retention,
        &crate::data::proof::PartitionScopeSet::default(),
        &mut crate::logic::evaluation::EvaluationWork::Ordinary,
    )
    .unwrap()
    .expect("cold intent");
    assert_eq!(
        intent.labels.len(),
        crate::data::trace::COLD_ARTIFACT_INTENT_LABEL_LIMIT
    );
    assert_eq!(intent.labels.as_slice(), &["a", "b", "c", "d"]);
}

mod comparison_work;

mod artifact_work;
