use super::{
    UiMountedAppearanceProjectionSelection, UiMountedProjectionBuild, UiMountedProjectionDenial,
};
use crate::runtime::appearance::{UiAppearanceInvalidationBatch, UiAppearanceInvalidationInput};

#[cfg(test)]
mod tests;

pub(super) fn finish(
    state: &crate::mounting::UiMountedIdentityState,
    requested_surfaces: &[worth_ui_host_contract::UiSemanticSurfaceIdentity],
    input: Option<UiAppearanceInvalidationInput<'_>>,
    build: &mut UiMountedProjectionBuild,
    selection: &mut UiMountedAppearanceProjectionSelection,
    portal_geometry_changes: &[worth_ui_host_contract::UiMountedInstanceIdentity],
    changes: &crate::mounting::UiMountedProjectionChangeSnapshot,
) -> Result<Option<UiAppearanceInvalidationBatch>, UiMountedProjectionDenial> {
    let Some(input) = input else {
        return Ok(None);
    };
    // The projection build supplies candidates, including allocation dependents
    // and both sides of a Portal transition. It is not a new consumer registry.
    let (candidates, probes) = changed_candidates(
        state,
        requested_surfaces,
        build,
        selection,
        portal_geometry_changes,
        changes,
    )?;
    let geometry =
        UiAppearanceInvalidationBatch::mounted_geometry(input.index, candidates.into_iter());
    build.cost.considered = build
        .cost
        .considered
        .checked_add(build.presentation_changed_instances.len())
        .ok_or(UiMountedProjectionDenial::CostCounterOverflow)?;
    build.cost.index_entries = build
        .cost
        .index_entries
        .checked_add(probes)
        .ok_or(UiMountedProjectionDenial::CostCounterOverflow)?;
    if geometry.selected_count() == 0 {
        return Ok(input.pending);
    }
    let addition =
        UiMountedAppearanceProjectionSelection::derive(state, requested_surfaces, Some(&geometry))
            .ok_or(UiMountedProjectionDenial::CostCounterOverflow)?;
    let mut effective = geometry;
    if let Some(pending) = input.pending {
        if pending.selected_count() == 0 {
            effective = effective.with_revision(pending.revision());
        } else {
            effective.merge(pending);
        }
    }
    selection
        .merge_physical_input_selection(addition, &effective)
        .ok_or(UiMountedProjectionDenial::CostCounterOverflow)?;
    Ok(Some(effective))
}

fn changed_candidates(
    state: &crate::mounting::UiMountedIdentityState,
    requested: &[worth_ui_host_contract::UiSemanticSurfaceIdentity],
    build: &UiMountedProjectionBuild,
    selection: &UiMountedAppearanceProjectionSelection,
    portal: &[worth_ui_host_contract::UiMountedInstanceIdentity],
    changes: &crate::mounting::UiMountedProjectionChangeSnapshot,
) -> Result<
    (
        Vec<(
            crate::graph::UiGraphNodeIdentity,
            worth_ui_host_contract::UiMountedInstanceIdentity,
        )>,
        usize,
    ),
    UiMountedProjectionDenial,
> {
    let mut candidates = Vec::new();
    let mut work = 0usize;
    let mut changed = build.presentation_changed_instances.to_vec();
    changed.extend(changes.appearance_input_changed_instances());
    changed.sort_unstable();
    changed.dedup();
    for instance in changed {
        if selection
            .selected_instances()
            .binary_search(&instance)
            .is_ok()
        {
            continue;
        }
        let (node, probes) = build.semantic.node_with_probes(instance);
        work = work
            .checked_add(probes)
            .ok_or(UiMountedProjectionDenial::CostCounterOverflow)?;
        let Some(node) = node else {
            continue;
        };
        let receipt = node.receipt();
        if !requested.contains(&receipt.semantic_surface()) {
            continue;
        }
        if portal.binary_search(&instance).is_err() && !changes.appearance_input_changed(instance) {
            if let Some(previous) = state.current_projection() {
                let (same, probes) =
                    same_geometry_input(previous.semantic_projection(), &build.semantic, node);
                work = work
                    .checked_add(probes)
                    .ok_or(UiMountedProjectionDenial::CostCounterOverflow)?;
                if same {
                    continue;
                }
            } else if let Some(previous) = state.appearance_predecessor() {
                let (surface, probes) = build
                    .semantic
                    .surface_for_with_probes(receipt.semantic_surface());
                work = work
                    .checked_add(probes)
                    .ok_or(UiMountedProjectionDenial::CostCounterOverflow)?;
                if let Some(surface) = surface {
                    let input =
                        super::super::appearance::UiMountedAppearanceGeometryInput::from_node(
                            node,
                            surface.binding,
                        );
                    let (same, probes) = previous.matches_geometry_input(instance, &input);
                    work = work
                        .checked_add(probes)
                        .ok_or(UiMountedProjectionDenial::CostCounterOverflow)?;
                    if same {
                        continue;
                    }
                }
            }
        }
        candidates.push((receipt.graph_node(), instance));
    }
    Ok((candidates, work))
}

fn same_geometry_input(
    predecessor: &super::UiMountedSemanticProjection,
    successor: &super::UiMountedSemanticProjection,
    node: &super::super::frame_storage::UiMountedProjectionNodeRecord,
) -> (bool, usize) {
    let receipt = node.receipt();
    let (previous, mut probes) = predecessor.node_with_probes(receipt.mounted_instance());
    let Some(previous) = previous else {
        return (false, probes);
    };
    if previous.receipt().semantic_surface() != receipt.semantic_surface() {
        return (false, probes);
    }
    let (old_surface, old_probes) = predecessor.surface_for_with_probes(receipt.semantic_surface());
    let (new_surface, new_probes) = successor.surface_for_with_probes(receipt.semantic_surface());
    probes += old_probes + new_probes;
    let (Some(old_surface), Some(new_surface)) = (old_surface, new_surface) else {
        return (false, probes);
    };
    use super::super::appearance::UiMountedAppearanceGeometryInput;
    (
        UiMountedAppearanceGeometryInput::from_node(previous, old_surface.binding)
            == UiMountedAppearanceGeometryInput::from_node(node, new_surface.binding),
        probes,
    )
}
