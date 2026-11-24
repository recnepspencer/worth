use std::sync::Arc;

use crate::merge::data::{
    AspectComparisonState, LoweredMergePlan, MergeConflictClassification, MergeConflictDigestBasis,
    MergeVisibilityEvidence, MergeVisibilityEvidenceKind,
};

pub(super) fn merge_conflict_digest_basis(plan: &LoweredMergePlan) -> MergeConflictDigestBasis {
    MergeConflictDigestBasis {
        records: Arc::from(
            plan.classifications
                .iter()
                .map(|classification| classification.record.clone())
                .collect::<Vec<_>>(),
        ),
        classes: Arc::from(
            plan.classifications
                .iter()
                .map(|classification| classification.class)
                .collect::<Vec<_>>(),
        ),
        validated_schema_correspondence: Arc::from(
            plan.classifications
                .iter()
                .map(|classification| classification.validated_schema_correspondence)
                .collect::<Vec<_>>(),
        ),
        strategy_conflict_classes: Arc::from(
            plan.classifications
                .iter()
                .map(|classification| {
                    classification
                        .strategy_evidence
                        .as_ref()
                        .map(|evidence| evidence.class)
                })
                .collect::<Vec<_>>(),
        ),
        source_strategy_descriptors: Arc::from(
            plan.classifications
                .iter()
                .map(source_strategy_descriptors_for_digest)
                .collect::<Vec<_>>(),
        ),
        target_strategy_descriptors: Arc::from(
            plan.classifications
                .iter()
                .map(target_strategy_descriptors_for_digest)
                .collect::<Vec<_>>(),
        ),
        relation_evidence: Arc::from(
            plan.classifications
                .iter()
                .map(|classification| classification.relation_evidence.clone())
                .collect::<Vec<_>>(),
        ),
        source_visibility_evidence: Arc::from(
            plan.classifications
                .iter()
                .map(|classification| classification.source_visibility_evidence.clone())
                .collect::<Vec<_>>(),
        ),
        target_visibility_evidence: Arc::from(
            plan.classifications
                .iter()
                .map(|classification| classification.target_visibility_evidence.clone())
                .collect::<Vec<_>>(),
        ),
        base_visibility_evidence: Arc::from(
            plan.classifications
                .iter()
                .map(normalized_base_visibility_evidence_for_digest)
                .collect::<Vec<_>>(),
        ),
        aspect_evidence_keys: Arc::from(
            plan.classifications
                .iter()
                .map(aspect_evidence_keys_for_digest)
                .collect::<Vec<_>>(),
        ),
        aspect_evidence_comparisons: Arc::from(
            plan.classifications
                .iter()
                .map(aspect_evidence_comparisons_for_digest)
                .collect::<Vec<_>>(),
        ),
    }
}

fn source_strategy_descriptors_for_digest(
    classification: &MergeConflictClassification,
) -> Arc<[crate::commit_strategies::data::StrategyMergeDescriptor]> {
    Arc::from(
        classification
            .strategy_evidence
            .as_ref()
            .map(|evidence| evidence.source_descriptors.to_vec())
            .unwrap_or_default(),
    )
}

fn target_strategy_descriptors_for_digest(
    classification: &MergeConflictClassification,
) -> Arc<[crate::commit_strategies::data::StrategyMergeDescriptor]> {
    Arc::from(
        classification
            .strategy_evidence
            .as_ref()
            .map(|evidence| evidence.target_descriptors.to_vec())
            .unwrap_or_default(),
    )
}

fn aspect_evidence_keys_for_digest(
    classification: &MergeConflictClassification,
) -> Arc<[worth_foundational::facade::AspectKey]> {
    Arc::from(
        classification
            .aspect_evidence
            .iter()
            .map(|evidence| evidence.aspect_key.clone())
            .collect::<Vec<_>>(),
    )
}

fn aspect_evidence_comparisons_for_digest(
    classification: &MergeConflictClassification,
) -> Arc<[AspectComparisonState]> {
    Arc::from(
        classification
            .aspect_evidence
            .iter()
            .map(|evidence| evidence.comparison)
            .collect::<Vec<_>>(),
    )
}

fn normalized_base_visibility_evidence_for_digest(
    classification: &MergeConflictClassification,
) -> MergeVisibilityEvidence {
    let mut evidence = classification.base_visibility_evidence.clone();
    if evidence.kind == MergeVisibilityEvidenceKind::BaseHistoricalWindow
        && classification.base_record_visible
        && classification.source_record_visible
        && classification.target_record_visible
    {
        evidence.kind = MergeVisibilityEvidenceKind::BaseResolvedViewLookup;
    }
    evidence
}
