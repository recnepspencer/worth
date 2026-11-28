use std::collections::BTreeMap;

use worth_signal::facade::adapters::{ArtifactMergeAction, BranchMergeResult};

use crate::expression::model::SignalValue;
use crate::runtime::summaries::{AspectVersionSummary, RuntimeStoreSnapshot, StoredSourceSnapshot};

use super::state::BranchRuntimeMetadata;

pub(super) fn merge_branch_metadata(
    target: &BranchRuntimeMetadata,
    source: &BranchRuntimeMetadata,
) -> BranchRuntimeMetadata {
    let mut merged = target.clone();
    for (node, id) in &source.nodes_by_id {
        merged
            .nodes_by_id
            .entry(*node)
            .or_insert_with(|| id.clone());
    }
    for (id, entry) in &source.catalog {
        merged
            .catalog
            .entry(id.clone())
            .or_insert_with(|| entry.clone());
    }
    for (family_id, grid) in &source.dense_grids {
        merged
            .dense_grids
            .entry(family_id.clone())
            .or_insert_with(|| grid.clone());
    }
    merged
}

pub(super) fn merge_branch_store(
    target: &RuntimeStoreSnapshot,
    source: &RuntimeStoreSnapshot,
    source_metadata: &BranchRuntimeMetadata,
    merged_metadata: &BranchRuntimeMetadata,
    result: &BranchMergeResult,
) -> RuntimeStoreSnapshot {
    let mut merged = target.clone();
    let source_sources = source
        .sources
        .iter()
        .map(|entry| (entry.id.clone(), entry))
        .collect::<BTreeMap<_, _>>();
    let mut merged_sources = merged
        .sources
        .iter()
        .map(|entry| (entry.id.clone(), entry.clone()))
        .collect::<BTreeMap<_, _>>();
    for record in &result.records {
        let should_adopt = matches!(
            record.action,
            ArtifactMergeAction::Adopted
                | ArtifactMergeAction::Replaced
                | ArtifactMergeAction::IntroducedIntoTarget
                | ArtifactMergeAction::EquivalentUnchanged
        );
        if !should_adopt {
            continue;
        }
        let Some(source_id) = source_metadata.nodes_by_id.get(&record.source_node) else {
            continue;
        };
        let Some(source_value) = source_sources.get(source_id) else {
            continue;
        };
        let target_id = record
            .target_node
            .and_then(|node| merged_metadata.nodes_by_id.get(&node).cloned())
            .unwrap_or_else(|| source_id.clone());
        merged_sources.insert(
            target_id.clone(),
            StoredSourceSnapshot {
                id: target_id,
                value: source_value.value.clone(),
                version: source_value.version,
                produces_aspects: None,
                aspect_versions: vec![AspectVersionSummary {
                    aspect: super::DEFAULT_ASPECT.id(),
                    version: source_value.version,
                }],
            },
        );
    }
    merged.sources = merged_sources.into_values().collect();
    for recipe in &mut merged.recipes {
        recipe.value = SignalValue::Null;
        recipe.initialized = false;
        recipe.output_identity = None;
    }
    merged
}
