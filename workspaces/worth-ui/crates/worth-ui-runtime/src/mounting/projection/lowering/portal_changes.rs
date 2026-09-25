use super::UiMountedSemanticProjection;
use worth_ui_host_contract::UiMountedInstanceIdentity;

/// Position changes can alter the issued Portal order. Preserve both old and
/// new participants, but unchanged neighbors do not become dirty merely because
/// another overlay changed.
pub(in crate::mounting::projection) fn changed_owners(
    predecessor: Option<&super::super::UiMountedProjectionFrame>,
    successor: &[crate::mounting::UiMountedPortalOverlayProjectionInput],
) -> Vec<UiMountedInstanceIdentity> {
    let predecessor = predecessor
        .map(super::super::UiMountedProjectionFrame::portal_overlay_inputs)
        .unwrap_or(&[]);
    let mut changed = Vec::new();
    for (index, old) in predecessor.iter().enumerate() {
        if successor
            .get(index)
            .is_none_or(|new| !old.same_mounted_projection_meaning(*new))
        {
            changed.push(old.owner());
        }
    }
    for (index, new) in successor.iter().enumerate() {
        if predecessor
            .get(index)
            .is_none_or(|old| !old.same_mounted_projection_meaning(*new))
        {
            changed.push(new.owner());
        }
    }
    changed.sort_unstable();
    changed.dedup();
    changed
}

/// Marks every Portal owner whose placement or order this frame changed, and
/// every child presented inside one, as changed presentation. Returns those
/// instances; none means no Portal changed.
pub(super) fn mark_changed(
    build: &mut super::UiMountedProjectionBuild,
    predecessor: Option<&super::super::UiMountedProjectionFrame>,
    overlays: &[crate::mounting::UiMountedPortalOverlayProjectionInput],
) -> Result<Vec<UiMountedInstanceIdentity>, super::UiMountedProjectionDenial> {
    let owners = changed_owners(predecessor, overlays);
    if owners.is_empty() {
        return Ok(owners);
    }
    let (children, work) = children_in_transition(
        predecessor.map(super::super::UiMountedProjectionFrame::semantic_projection),
        &build.semantic,
        &owners,
    );
    build.cost.index_entries = build
        .cost
        .index_entries
        .checked_add(work)
        .ok_or(super::UiMountedProjectionDenial::CostCounterOverflow)?;
    let mut geometry = children;
    geometry.extend_from_slice(&owners);
    geometry.sort_unstable();
    geometry.dedup();
    let mut changed = build.presentation_changed_instances.to_vec();
    changed.extend_from_slice(&geometry);
    changed.sort_unstable();
    changed.dedup();
    build.presentation_changed_instances = changed.into();
    Ok(geometry)
}

pub(in crate::mounting::projection) fn children_in_transition(
    predecessor: Option<&UiMountedSemanticProjection>,
    successor: &UiMountedSemanticProjection,
    owners: &[UiMountedInstanceIdentity],
) -> (Vec<UiMountedInstanceIdentity>, usize) {
    let (mut selected, mut work) = successor.portal_children_for_owners(owners);
    if let Some(previous) = predecessor {
        let (retired, previous_work) = previous.portal_children_for_owners(owners);
        selected.extend(retired);
        work = work
            .checked_add(previous_work)
            .expect("Portal selection work fits address space");
    }
    selected.sort_unstable();
    selected.dedup();
    (selected, work)
}
