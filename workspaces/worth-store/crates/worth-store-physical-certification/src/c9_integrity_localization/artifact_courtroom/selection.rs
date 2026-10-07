//! Mandatory family and operator selection from a production-issued inventory.
use super::{ArtifactInventory, ProductionWorldProfile};
use crate::c9_integrity_localization::artifact_edit::ArtifactOperator;
use std::collections::BTreeSet;

pub(super) fn terminal_truncate_target(
    inventory: &ArtifactInventory,
    index: usize,
    operator: ArtifactOperator,
) -> usize {
    let target = &inventory.granules[index];
    if operator != ArtifactOperator::Truncate
        || !matches!(
            target.family,
            "inline_page" | "extent_manifest" | "extent_chunk"
        )
    {
        return index;
    }
    // A file truncation must target its final selected frame; an arena
    // manifest truncation also removes that extent's chunks.
    inventory
        .granules
        .iter()
        .enumerate()
        .filter(|(_, candidate)| candidate.path == target.path && candidate.family == target.family)
        .max_by_key(|(_, candidate)| candidate.offset())
        .unwrap()
        .0
}

pub(super) fn rows(
    inventory: &ArtifactInventory,
    profile: ProductionWorldProfile,
    selection: Option<&str>,
) -> Vec<(
    Option<usize>,
    crate::c9_integrity_localization::artifact_edit::ArtifactOperator,
)> {
    let mut families = BTreeSet::new();
    let selected = inventory
        .granules
        .iter()
        .enumerate()
        .filter_map(|(index, granule)| {
            let applicable = if selection == Some("journals") {
                granule.grammar
                    != crate::c9_integrity_localization::artifact_inventory::FrameGrammar::Common
            } else if let Some(family) = selection {
                granule.family == family
            } else {
                profile == ProductionWorldProfile::Primary16KiB || granule.family == "inline_page"
            };
            (applicable && families.insert(granule.family)).then_some(index)
        })
        .collect::<Vec<_>>();
    let rows = std::iter::once((
        None,
        crate::c9_integrity_localization::artifact_edit::ArtifactOperator::CoveredByte,
    ))
    .chain(selected.into_iter().flat_map(|index| {
        let mut operators =
            crate::c9_integrity_localization::artifact_edit::operators(&inventory.granules[index]);
        if matches!(
            inventory.granules[index].family,
            "inline_page" | "extent_chunk"
        ) {
            operators.extend([
                crate::c9_integrity_localization::artifact_edit::ArtifactOperator::Remove,
                crate::c9_integrity_localization::artifact_edit::ArtifactOperator::Duplicate,
            ]);
        }
        operators
            .into_iter()
            .map(move |operator| (Some(index), operator))
    }));
    if let Some(family) = selection.filter(|name| *name != "journals") {
        assert_eq!(families, BTreeSet::from([family]));
    } else if profile == ProductionWorldProfile::Primary16KiB && selection.is_none() {
        assert_eq!(
            families,
            BTreeSet::from([
                "bootstrap_catalog",
                "current_root_selector",
                "previous_root_selector",
                "root_manifest",
                "root_routing_block",
                "segment_membership_block",
                "inline_page",
                "extent_manifest",
                "extent_chunk",
                "free_space_header",
                "free_space_membership_block",
                "wal_frame",
                "checkpoint_stream_header",
                "checkpoint_dirty_basis",
                "checkpoint_binding_compaction",
                "checkpoint_binding",
                "checkpoint_footer",
            ]),
            "every mandatory production family must enter the matrix"
        );
    } else if selection == Some("journals") {
        for family in [
            "wal_frame",
            "checkpoint_stream_header",
            "checkpoint_dirty_basis",
            "checkpoint_binding_compaction",
            "checkpoint_binding",
            "checkpoint_footer",
        ] {
            assert!(
                families.contains(family),
                "production matrix must select {family}"
            );
        }
    }
    rows.collect()
}
