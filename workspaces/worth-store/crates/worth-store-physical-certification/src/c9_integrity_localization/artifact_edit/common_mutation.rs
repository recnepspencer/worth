use super::{common_render, enclosures, ArtifactInventory, ArtifactOperator as Operator};
use std::path::{Path, PathBuf};

fn refreshes(operator: Operator) -> bool {
    matches!(
        operator,
        Operator::Checksum
            | Operator::Length
            | Operator::ScopeSubstitution
            | Operator::Pointer
            | Operator::EnvelopeVersion
            | Operator::RecordVersion
    )
}
pub(super) fn apply(
    root: &Path,
    baseline: &Path,
    inventory: &ArtifactInventory,
    index: usize,
    operator: Operator,
) {
    let path = root.join(&inventory.granules[index].path);
    std::fs::write(path, common_render(baseline, inventory, index, operator)).unwrap();
    if refreshes(operator) {
        let edges = enclosures::graph(inventory, baseline);
        enclosures::refresh(root, inventory, &edges, index);
    }
}
pub(in super::super) fn enclosing_paths(
    inventory: &ArtifactInventory,
    baseline: &Path,
    index: usize,
    operator: Operator,
) -> Vec<PathBuf> {
    if !refreshes(operator) || inventory.granules[index].grammar != super::FrameGrammar::Common {
        return Vec::new();
    }
    enclosures::affected(&enclosures::graph(inventory, baseline), index)
        .into_iter()
        .filter(|candidate| *candidate != index)
        .map(|candidate| inventory.granules[candidate].path.clone())
        .collect()
}
pub(in super::super) fn audit_enclosures(
    root: &Path,
    baseline: &Path,
    inventory: &ArtifactInventory,
    index: usize,
    operator: Operator,
) {
    if inventory.granules[index].grammar != super::FrameGrammar::Common {
        return;
    }
    if refreshes(operator) {
        enclosures::audit(
            root,
            baseline,
            inventory,
            &enclosures::graph(inventory, baseline),
            index,
        );
    }
    // The named target audit separately checks semantic field relations. Exact donor
    // equality additionally prevents S from smuggling arbitrary same-width bytes.
    if operator == Operator::ScopeSubstitution && inventory.granules[index].family == "inline_page"
    {
        let actual = std::fs::read(root.join(&inventory.granules[index].path)).unwrap();
        assert_eq!(actual, common_render(baseline, inventory, index, operator));
    }
}
