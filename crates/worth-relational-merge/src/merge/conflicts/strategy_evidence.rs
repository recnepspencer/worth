use std::collections::BTreeMap;
use std::sync::Arc;

use crate::merge::data::{StrategyConflictClass, StrategyConflictEvidence};

pub(super) fn overlapping_strategy_intent_diverges(
    evidence: &StrategyConflictEvidence,
    aspect_evidence: &[crate::merge::data::AspectConflictEvidence],
) -> bool {
    evidence.source_descriptors.iter().any(|source| {
        evidence.target_descriptors.iter().any(|target| {
            source.intent_scope_targets().iter().any(|source_field| {
                target
                    .intent_scope_targets()
                    .iter()
                    .filter(|target_field| *target_field == source_field)
                    .any(|_| {
                        aspect_evidence
                            .iter()
                            .find(|aspect| &aspect.aspect_key == source_field.aspect().aspect_key())
                            .is_none_or(|aspect| {
                                aspect.comparison
                                    != crate::merge::data::AspectComparisonState::Equal
                            })
                    })
            })
        })
    })
}

pub(super) fn strategy_conflict_evidence(
    source_delta: Option<&crate::merge::data::BranchTouchedRecordDelta>,
    target_delta: Option<&crate::merge::data::BranchTouchedRecordDelta>,
) -> Option<StrategyConflictEvidence> {
    let source_descriptors = strategy_descriptors_for_delta(source_delta);
    let target_descriptors = strategy_descriptors_for_delta(target_delta);
    if source_descriptors.is_empty() && target_descriptors.is_empty() {
        return None;
    }
    let class = match (source_descriptors.is_empty(), target_descriptors.is_empty()) {
        (false, false) => {
            let any_same_descriptor = source_descriptors.iter().any(|source_descriptor| {
                target_descriptors.iter().any(|target_descriptor| {
                    target_descriptor.descriptor_digest() == source_descriptor.descriptor_digest()
                })
            });
            let any_same_scope = source_descriptors.iter().any(|source_descriptor| {
                target_descriptors.iter().any(|target_descriptor| {
                    strategy_scope_overlaps(source_descriptor, target_descriptor)
                })
            });
            if any_same_descriptor && any_same_scope {
                StrategyConflictClass::SameStrategyDivergentOutput
            } else if any_same_scope {
                StrategyConflictClass::DifferentStrategyOverlappingIntent
            } else {
                return None;
            }
        }
        (false, true) => StrategyConflictClass::SourceStrategyOnly,
        (true, false) => StrategyConflictClass::TargetStrategyOnly,
        (true, true) => return None,
    };
    Some(StrategyConflictEvidence {
        class,
        source_commit_ids: Arc::from(
            source_delta
                .map(|delta| delta.commit_ids().collect::<Vec<_>>())
                .unwrap_or_default(),
        ),
        target_commit_ids: Arc::from(
            target_delta
                .map(|delta| delta.commit_ids().collect::<Vec<_>>())
                .unwrap_or_default(),
        ),
        source_descriptors: Arc::from(source_descriptors),
        target_descriptors: Arc::from(target_descriptors),
    })
}

fn strategy_descriptors_for_delta(
    delta: Option<&crate::merge::data::BranchTouchedRecordDelta>,
) -> Vec<crate::commit_strategies::data::StrategyMergeDescriptor> {
    let Some(delta) = delta else {
        return Vec::new();
    };
    let mut latest_by_strategy =
        BTreeMap::<[u8; 32], crate::commit_strategies::data::StrategyMergeDescriptor>::new();
    for envelope in delta.envelopes.iter().rev() {
        let Some(strategy_artifacts) = envelope.strategy_artifacts.as_ref() else {
            continue;
        };
        let descriptor = strategy_artifacts.merge_descriptor().clone();
        latest_by_strategy
            .entry(descriptor.descriptor_digest().0)
            .or_insert(descriptor);
    }
    latest_by_strategy.into_values().collect()
}

fn strategy_scope_overlaps(
    source: &crate::commit_strategies::data::StrategyMergeDescriptor,
    target: &crate::commit_strategies::data::StrategyMergeDescriptor,
) -> bool {
    if source.intent_scope_digest() == target.intent_scope_digest() {
        return true;
    }
    source.intent_scope_targets().iter().any(|field| {
        target
            .intent_scope_targets()
            .iter()
            .any(|other| other == field)
    })
}
